<script setup lang="ts">
import { Background } from '@vue-flow/background'
import { Controls } from '@vue-flow/controls'
import { VueFlow, type DefaultEdge, type Edge, type Node, type NodeComponent, type NodeTypesObject } from '@vue-flow/core'
import { Link2, Plus, Save, Trash2 } from 'lucide-vue-next'
import { markRaw, computed, onMounted, reactive, ref, watch } from 'vue'
import { useRouter } from 'vue-router'

import DeviceNode from '@/components/topology/DeviceNode.vue'
import Modal from '@/components/ui/Modal.vue'
import { api } from '@/lib/ipc'
import {
  linkTypeColor,
  linkTypeLabel,
  linkTypeMeta,
} from '@/lib/format'
import type { Device, LayoutNode, LinkType } from '@/lib/types'
import { useAppStore } from '@/stores/app'
import { useThemeStore } from '@/stores/theme'

import '@vue-flow/core/dist/style.css'
import '@vue-flow/core/dist/theme-default.css'
import '@vue-flow/controls/dist/style.css'

const store = useAppStore()
const theme = useThemeStore()
const router = useRouter()

const nodes = ref<Node[]>([])
const edges = ref<Edge[]>([])
const nodeTypes: NodeTypesObject = {
  device: markRaw(DeviceNode) as unknown as NodeComponent,
}
const patternColor = computed(() =>
  theme.resolved === 'light' ? '#e2e8f0' : '#1e293b',
)

const linkOpen = ref(false)
const saving = ref(false)
const linking = ref(false)
const linkError = ref('')

const linkForm = reactive({
  srcDeviceId: '',
  srcPort: '',
  dstDeviceId: '',
  dstPort: '',
  linkType: 'ethernet' as LinkType,
  speed: '1G',
  note: '',
})

const linkTypeOptions = Object.entries(linkTypeMeta).map(([value, meta]) => ({
  value: value as LinkType,
  label: meta.label,
}))

function nodeId(device: Device): string {
  return String(device.id)
}

function autoPosition(device: Device, fallback: number): { x: number; y: number } {
  const racks = [...store.racks].sort((a, b) => a.name.localeCompare(b.name))
  const column = device.rackId
    ? Math.max(0, racks.findIndex((rack) => rack.id === device.rackId))
    : racks.length
  const peers = store.devices
    .filter((peer) => peer.rackId === device.rackId)
    .sort((a, b) => (b.rackPosition ?? 0) - (a.rackPosition ?? 0))
  const row = peers.findIndex((peer) => peer.id === device.id)
  return {
    x: 60 + column * 280,
    y: 60 + (row >= 0 ? row : fallback) * 110,
  }
}

function rebuildEdges(): void {
  const next: Edge[] = []

  for (const link of store.links) {
    const edge: DefaultEdge = {
      id: `link-${link.id}`,
      source: String(link.srcDeviceId),
      target: String(link.dstDeviceId),
      type: 'smoothstep',
      animated: link.linkType === 'power',
      label:
        [link.linkType ? linkTypeLabel(link.linkType) : null, link.speed]
          .filter(Boolean)
          .join(' · ') || undefined,
      style: {
        stroke: linkTypeColor(link.linkType),
        strokeWidth: 1.6,
      },
      labelStyle: { fill: '#94a3b8', fontSize: '9px' },
      labelBgStyle: { fill: '#0f172a', fillOpacity: 0.9 },
    }
    next.push(edge)
  }

  edges.value = next
}

async function loadGraph(): Promise<void> {
  let saved: LayoutNode[] = []
  try {
    saved = await api.loadLayout('default')
  } catch {
    saved = []
  }
  const savedMap = new Map(saved.map((item) => [item.deviceId, item]))
  const next: Node[] = []

  store.devices.forEach((device, index) => {
    const position = savedMap.get(device.id)
      ? { x: savedMap.get(device.id)!.x, y: savedMap.get(device.id)!.y }
      : autoPosition(device, index)

    const node: Node = {
      id: nodeId(device),
      type: 'device',
      position,
      data: { device },
    }
    next.push(node)
  })

  nodes.value = next

  rebuildEdges()
}

onMounted(() => {
  void loadGraph()
})

watch(
  () => store.links.length,
  () => rebuildEdges(),
)

watch(
  () => store.devices.map((device) => `${device.id}:${device.name}`).join('|'),
  () => {
    void loadGraph()
  },
)

async function saveLayout(): Promise<void> {
  saving.value = true
  try {
    const current = nodes.value as unknown as Array<{
      id: string
      position: { x: number; y: number }
    }>
    const layout = current.map((node) => ({
      deviceId: Number(node.id),
      x: node.position.x,
      y: node.position.y,
    }))
    await api.saveLayout('default', layout)
    store.notify('拓扑布局已保存')
  } catch (caught) {
    store.notify(caught instanceof Error ? caught.message : '保存布局失败')
  } finally {
    saving.value = false
  }
}

function openLinkModal(): void {
  linkForm.srcDeviceId = ''
  linkForm.srcPort = ''
  linkForm.dstDeviceId = ''
  linkForm.dstPort = ''
  linkForm.linkType = 'ethernet'
  linkForm.speed = '1G'
  linkForm.note = ''
  linkError.value = ''
  linkOpen.value = true
}

async function submitLink(): Promise<void> {
  linkError.value = ''
  if (!linkForm.srcDeviceId || !linkForm.dstDeviceId) {
    linkError.value = '请选择两端设备'
    return
  }
  if (linkForm.srcDeviceId === linkForm.dstDeviceId) {
    linkError.value = '不能连接同一台设备'
    return
  }

  linking.value = true
  try {
    await store.createLink({
      srcDeviceId: Number(linkForm.srcDeviceId),
      srcPort: linkForm.srcPort.trim() || null,
      dstDeviceId: Number(linkForm.dstDeviceId),
      dstPort: linkForm.dstPort.trim() || null,
      linkType: linkForm.linkType,
      speed: linkForm.speed.trim() || null,
      note: linkForm.note.trim() || null,
    })
    store.notify('连接已创建')
    linkOpen.value = false
  } catch (caught) {
    linkError.value = caught instanceof Error ? caught.message : '创建连接失败'
  } finally {
    linking.value = false
  }
}

async function removeLink(id: number): Promise<void> {
  if (!window.confirm('确定删除这条连接吗？')) return
  try {
    await store.deleteLink(id)
    store.notify('连接已删除')
  } catch (caught) {
    store.notify(caught instanceof Error ? caught.message : '删除失败')
  }
}

function deviceName(id: number): string {
  return store.deviceById.get(id)?.name ?? `#${id}`
}
</script>

<template>
  <div class="space-y-4">
    <div class="flex items-end justify-between">
      <div>
        <h2 class="text-base font-semibold tracking-tight text-slate-100">拓扑</h2>
        <p class="mt-1 text-xs text-slate-500">
          {{ store.devices.length }} 个节点 · {{ store.links.length }} 条连接 · 拖拽后记得保存布局
        </p>
      </div>
      <div class="flex gap-2">
        <button class="btn btn-ghost" @click="saveLayout" :disabled="saving">
          <Save class="h-3.5 w-3.5" />
          {{ saving ? '保存中…' : '保存布局' }}
        </button>
        <button class="btn btn-primary" @click="openLinkModal">
          <Plus class="h-3.5 w-3.5" />
          添加连接
        </button>
      </div>
    </div>

    <div v-if="store.devices.length" class="grid grid-cols-4 gap-4">
      <section class="card col-span-3 overflow-hidden">
        <div class="h-[calc(100vh-13rem)] min-h-[30rem] w-full">
          <VueFlow
            v-model:nodes="nodes"
            v-model:edges="edges"
            :node-types="nodeTypes"
            :fit-view-on-init="true"
            :min-zoom="0.25"
            :max-zoom="1.6"
            :default-edge-options="{ type: 'smoothstep' }"
          >
            <Background :pattern-color="patternColor" :gap="22" :size="1" />
            <Controls />
          </VueFlow>
        </div>
      </section>

      <aside class="card col-span-1 flex max-h-[calc(100vh-13rem)] flex-col p-4">
        <div class="flex items-center justify-between">
          <h3 class="text-xs font-semibold tracking-wide text-slate-200">连接</h3>
          <span class="text-[10px] text-slate-500">{{ store.links.length }} 条</span>
        </div>

        <div v-if="store.links.length" class="mt-3 space-y-2 overflow-y-auto pr-1">
          <div
            v-for="link in store.links"
            :key="link.id"
            class="group rounded-lg border border-white/5 bg-white/[0.02] p-2.5"
          >
            <div class="flex items-start justify-between gap-2">
              <div class="min-w-0">
                <p class="truncate text-[11px] text-slate-300">
                  {{ deviceName(link.srcDeviceId) }}
                  <span class="text-slate-600">→</span>
                  {{ deviceName(link.dstDeviceId) }}
                </p>
                <p class="mt-0.5 truncate text-[10px] text-slate-500">
                  {{ linkTypeLabel(link.linkType) }}
                  <template v-if="link.speed"> · {{ link.speed }}</template>
                  <template v-if="link.srcPort || link.dstPort">
                    · {{ link.srcPort ?? '—' }} → {{ link.dstPort ?? '—' }}
                  </template>
                </p>
              </div>
              <button
                class="rounded p-1 text-slate-600 opacity-0 transition hover:bg-rose-500/10 hover:text-rose-300 group-hover:opacity-100"
                title="删除连接"
                @click="removeLink(link.id)"
              >
                <Trash2 class="h-3 w-3" />
              </button>
            </div>
          </div>
        </div>

        <div v-else class="flex flex-1 flex-col items-center justify-center gap-2 py-10 text-center">
          <Link2 class="h-6 w-6 text-slate-600" />
          <p class="text-[11px] text-slate-500">还没有连接</p>
          <button class="btn btn-ghost mt-1" @click="openLinkModal">添加第一条</button>
        </div>

        <div class="mt-4 space-y-1.5 border-t border-white/5 pt-3">
          <p class="text-[10px] text-slate-500">图例</p>
          <div class="flex flex-wrap gap-x-4 gap-y-1">
            <span
              v-for="(meta, type) in linkTypeMeta"
              :key="type"
              class="flex items-center gap-1.5 text-[10px] text-slate-400"
            >
              <span class="h-1.5 w-4 rounded-full" :style="{ backgroundColor: meta.color }" />
              {{ meta.label }}
            </span>
          </div>
        </div>
      </aside>
    </div>

    <div v-else class="card flex flex-col items-center justify-center gap-3 px-6 py-20 text-center">
      <p class="text-sm font-medium text-slate-300">还没有可展示的设备</p>
      <p class="max-w-sm text-xs leading-5 text-slate-500">
        先在「设备」中添加服务器、交换机和连接关系，拓扑图会自动生成。
      </p>
      <button class="btn btn-primary" @click="router.push('/devices')">
        前往设备页面
      </button>
    </div>

    <Modal :open="linkOpen" title="添加连接" width="max-w-xl" @close="linkOpen = false">
      <form class="space-y-4" @submit.prevent="submitLink">
        <div class="grid grid-cols-2 gap-4">
          <div>
            <label class="label" for="link-src">源设备</label>
            <select id="link-src" v-model="linkForm.srcDeviceId" class="input">
              <option value="">请选择</option>
              <option v-for="device in store.devices" :key="device.id" :value="String(device.id)">
                {{ device.name }}
              </option>
            </select>
          </div>
          <div>
            <label class="label" for="link-src-port">源端口</label>
            <input
              id="link-src-port"
              v-model="linkForm.srcPort"
              class="input"
              placeholder="如 Gi0/1"
            />
          </div>
          <div>
            <label class="label" for="link-dst">目标设备</label>
            <select id="link-dst" v-model="linkForm.dstDeviceId" class="input">
              <option value="">请选择</option>
              <option v-for="device in store.devices" :key="device.id" :value="String(device.id)">
                {{ device.name }}
              </option>
            </select>
          </div>
          <div>
            <label class="label" for="link-dst-port">目标端口</label>
            <input
              id="link-dst-port"
              v-model="linkForm.dstPort"
              class="input"
              placeholder="如 Gi0/24"
            />
          </div>
          <div>
            <label class="label" for="link-type">连接类型</label>
            <select id="link-type" v-model="linkForm.linkType" class="input">
              <option v-for="option in linkTypeOptions" :key="option.value" :value="option.value">
                {{ option.label }}
              </option>
            </select>
          </div>
          <div>
            <label class="label" for="link-speed">速率</label>
            <input
              id="link-speed"
              v-model="linkForm.speed"
              class="input"
              placeholder="如 1G / 10G"
            />
          </div>
        </div>

        <div>
          <label class="label" for="link-note">备注</label>
          <input id="link-note" v-model="linkForm.note" class="input" />
        </div>

        <p
          v-if="linkError"
          class="rounded-lg border border-rose-500/20 bg-rose-500/10 px-3 py-2 text-xs text-rose-300"
        >
          {{ linkError }}
        </p>
      </form>

      <template #footer>
        <button class="btn btn-ghost" @click="linkOpen = false">取消</button>
        <button class="btn btn-primary" :disabled="linking" @click="submitLink">
          {{ linking ? '创建中…' : '创建连接' }}
        </button>
      </template>
    </Modal>
  </div>
</template>
