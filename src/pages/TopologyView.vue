<script setup lang="ts">
import { Background } from '@vue-flow/background'
import { Controls } from '@vue-flow/controls'
import {
  VueFlow,
  type Connection,
  type DefaultEdge,
  type Edge,
  type Node,
  type NodeComponent,
  type NodeMouseEvent,
  type NodeTypesObject,
} from '@vue-flow/core'
import { Link2, Plus, Save, Trash2 } from 'lucide-vue-next'
import { computed, markRaw, onMounted, reactive, ref, watch } from 'vue'
import { useRouter } from 'vue-router'

import CredentialChooserModal from '@/components/device/CredentialChooserModal.vue'
import DeviceFormModal from '@/components/device/DeviceFormModal.vue'
import DeviceNode from '@/components/topology/DeviceNode.vue'
import RackNode from '@/components/topology/RackNode.vue'
import TopologyNodeMenu from '@/components/topology/TopologyNodeMenu.vue'
import Modal from '@/components/ui/Modal.vue'
import { useDeviceConnect } from '@/composables/useDeviceConnect'
import {
  linkTypeColor,
  linkTypeLabel,
  linkTypeMeta,
} from '@/lib/format'
import { api } from '@/lib/ipc'
import type {
  ConnectionTypeSummary,
  CredentialAction,
  Device,
  LayoutNode,
  LinkType,
  Rack,
  RackLayoutNode,
} from '@/lib/types'
import { useAppStore } from '@/stores/app'
import { useThemeStore } from '@/stores/theme'

import '@vue-flow/core/dist/style.css'
import '@vue-flow/core/dist/theme-default.css'
import '@vue-flow/controls/dist/style.css'

const store = useAppStore()
const theme = useThemeStore()
const router = useRouter()

const {
  connectingKey,
  chooserOpen,
  chooser,
  connectionTypes,
  connect,
  chooseCredential,
  closeChooser,
} = useDeviceConnect()

/**
 * Container geometry. Racks contain devices, hosts contain their VMs; every
 * child is positioned relative to its parent and laid out automatically.
 */
const RACK_WIDTH = 380
const RACK_HEADER = 58
const RACK_INSET = 18
const RACK_PAD = 18
// Generous vertical gaps keep the in/out handles of neighbouring cards
// reachable when dragging a connection.
const RACK_GAP = 30
const RACK_COLUMN_GAP = 80
const RACK_EMPTY_HEIGHT = 130
const RACK_DEVICE_HEIGHT = 64

const HOST_WIDTH = 300
const HOST_HEADER = 72
const VM_HEIGHT = 62
const VM_GAP = 28
const VM_INSET = 14

const ROOT_DEVICE_HEIGHT = 92
const ROOT_STACK_GAP = 40

const nodes = ref<Node[]>([])
const edges = ref<Edge[]>([])
const nodeTypes: NodeTypesObject = {
  device: markRaw(DeviceNode) as unknown as NodeComponent,
  rack: markRaw(RackNode) as unknown as NodeComponent,
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

const menuOpen = ref(false)
const menuPosition = reactive({ x: 0, y: 0 })
const menuDevice = ref<Device | null>(null)
const menuTypes = computed(() =>
  menuDevice.value ? connectionTypes(menuDevice.value.id) : [],
)

const deviceFormOpen = ref(false)
const deviceFormDevice = ref<Device | null>(null)
const deviceFormTab = ref<'basic' | 'credentials'>('basic')

function nodeId(device: Device): string {
  return String(device.id)
}

function rackNodeId(rack: Rack): string {
  return `rack-${rack.id}`
}

function hostHeight(count: number): number {
  return HOST_HEADER + count * VM_HEIGHT + Math.max(0, count - 1) * VM_GAP + 18
}

/** Only physical servers can host VMs, and only VMs can have a host. */
function parentOf(device: Device): number | null {
  if (device.kind !== 'vm') return null
  const hostId = device.hostId
  if (!hostId || hostId === device.id) return null
  const host = store.deviceById.get(hostId)
  if (!host || host.kind !== 'physical_server') return null
  return hostId
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
  let savedDevices: LayoutNode[] = []
  let savedRacks: RackLayoutNode[] = []
  try {
    savedDevices = await api.loadLayout('default')
  } catch {
    savedDevices = []
  }
  try {
    savedRacks = await api.loadRackLayout('default')
  } catch {
    savedRacks = []
  }

  const savedDeviceMap = new Map(savedDevices.map((item) => [item.deviceId, item]))
  const savedRackMap = new Map(savedRacks.map((item) => [item.rackId, item]))

  // Keep in-memory positions so editing a device does not discard unsaved drags.
  const currentPositions = new Map<string, { x: number; y: number }>()
  const previousParents = new Map<string, string | null>()
  for (const node of nodes.value) {
    currentPositions.set(node.id, { x: node.position.x, y: node.position.y })
    previousParents.set(node.id, node.parentNode ?? null)
  }
  const wasRoot = (id: string) => previousParents.get(id) === null

  const racks = [...store.racks].sort((a, b) => a.name.localeCompare(b.name))
  const rackById = store.rackById

  const hostedByHost = new Map<number, Device[]>()
  const devicesByRack = new Map<number, Device[]>()
  const standalone: Device[] = []

  for (const device of store.devices) {
    const hostId = parentOf(device)
    if (hostId !== null) {
      const list = hostedByHost.get(hostId) ?? []
      list.push(device)
      hostedByHost.set(hostId, list)
      continue
    }

    const rackId =
      device.rackId && rackById.has(device.rackId) ? device.rackId : null
    if (rackId !== null) {
      const list = devicesByRack.get(rackId) ?? []
      list.push(device)
      devicesByRack.set(rackId, list)
    } else {
      standalone.push(device)
    }
  }

  for (const list of hostedByHost.values()) {
    list.sort((a, b) => a.name.localeCompare(b.name))
  }
  for (const list of devicesByRack.values()) {
    // Top U first, matching a rack elevation view.
    list.sort((a, b) => {
      const positionA = a.rackPosition ?? -1
      const positionB = b.rackPosition ?? -1
      if (positionA !== positionB) return positionB - positionA
      return a.name.localeCompare(b.name)
    })
  }
  standalone.sort((a, b) => a.name.localeCompare(b.name))

  const heightOf = (device: Device, inRack: boolean): number => {
    const count = hostedByHost.get(device.id)?.length ?? 0
    if (count) return hostHeight(count)
    return inRack ? RACK_DEVICE_HEIGHT : ROOT_DEVICE_HEIGHT
  }

  const rackHeight = (rackId: number): number => {
    const devices = devicesByRack.get(rackId) ?? []
    if (!devices.length) return RACK_EMPTY_HEIGHT
    const content = devices.reduce(
      (sum, device, index) =>
        sum + heightOf(device, true) + (index ? RACK_GAP : 0),
      0,
    )
    return RACK_HEADER + RACK_PAD * 2 + content
  }

  // Racks are roots laid out as a compact grid.
  const rackHeights = new Map(racks.map((rack) => [rack.id, rackHeight(rack.id)]))
  const columns = racks.length
    ? Math.max(1, Math.min(4, Math.ceil(Math.sqrt(racks.length))))
    : 0

  const rowHeights: number[] = []
  racks.forEach((rack, index) => {
    const row = Math.floor(index / Math.max(1, columns))
    rowHeights[row] = Math.max(rowHeights[row] ?? 0, rackHeights.get(rack.id) ?? 0)
  })
  const rowY: number[] = []
  let rowCursor = 60
  rowHeights.forEach((height, row) => {
    rowY[row] = rowCursor
    rowCursor += height + 80
  })

  const rackPositions = new Map<number, { x: number; y: number }>()
  racks.forEach((rack, index) => {
    const row = Math.floor(index / Math.max(1, columns))
    const column = index % Math.max(1, columns)
    rackPositions.set(rack.id, {
      x: 60 + column * (RACK_WIDTH + RACK_COLUMN_GAP),
      y: rowY[row] ?? 60,
    })
  })

  // Devices without a rack are stacked in a column right of the rack grid.
  const standaloneX = racks.length
    ? 60 + Math.max(1, columns) * (RACK_WIDTH + RACK_COLUMN_GAP)
    : 60
  let standaloneY = 60
  const standalonePositions = new Map<number, { x: number; y: number }>()
  for (const device of standalone) {
    standalonePositions.set(device.id, { x: standaloneX, y: standaloneY })
    standaloneY += heightOf(device, false) + ROOT_STACK_GAP
  }

  const next: Node[] = []

  // Parents must be added before their children, otherwise Vue Flow reports a
  // missing parent and skips the nesting.
  for (const rack of racks) {
    const id = rackNodeId(rack)
    const devices = devicesByRack.get(rack.id) ?? []
    const saved = savedRackMap.get(rack.id)
    const previous = wasRoot(id) ? currentPositions.get(id) : undefined
    const usedU = devices.reduce(
      (sum, device) => sum + (device.rackPosition ? device.rackHeight : 0),
      0,
    )

    next.push({
      id,
      type: 'rack',
      position:
        previous ??
        (saved ? { x: saved.x, y: saved.y } : rackPositions.get(rack.id) ?? { x: 60, y: 60 }),
      draggable: true,
      connectable: false,
      width: RACK_WIDTH,
      height: rackHeights.get(rack.id) ?? RACK_EMPTY_HEIGHT,
      data: { rack, deviceCount: devices.length, usedU },
    })
  }

  for (const device of standalone) {
    const id = nodeId(device)
    const count = hostedByHost.get(device.id)?.length ?? 0
    const saved = savedDeviceMap.get(device.id)
    const previous = wasRoot(id) ? currentPositions.get(id) : undefined

    const node: Node = {
      id,
      type: 'device',
      position:
        previous ??
        (saved
          ? { x: saved.x, y: saved.y }
          : standalonePositions.get(device.id) ?? { x: 60, y: 60 }),
      data: {
        device,
        childCount: count,
        variant: count ? 'host' : 'device',
      },
    }

    if (count) {
      node.width = HOST_WIDTH
      node.height = hostHeight(count)
    }

    next.push(node)
  }

  for (const rack of racks) {
    const devices = devicesByRack.get(rack.id) ?? []
    let y = RACK_HEADER + RACK_PAD
    const width = RACK_WIDTH - RACK_INSET * 2

    for (const device of devices) {
      const count = hostedByHost.get(device.id)?.length ?? 0
      const height = heightOf(device, true)

      next.push({
        id: nodeId(device),
        type: 'device',
        parentNode: rackNodeId(rack),
        // Rack contents are arranged automatically and move with the rack.
        draggable: false,
        width,
        height,
        position: { x: RACK_INSET, y },
        data: {
          device,
          childCount: count,
          variant: count ? 'host' : 'racked',
        },
      })

      y += height + RACK_GAP
    }
  }

  for (const device of store.devices) {
    const children = hostedByHost.get(device.id)
    if (!children?.length) continue

    const inRack = Boolean(device.rackId && rackById.has(device.rackId))
    const width = inRack ? RACK_WIDTH - RACK_INSET * 2 : HOST_WIDTH

    children.forEach((child, index) => {
      next.push({
        id: nodeId(child),
        type: 'device',
        parentNode: nodeId(device),
        // Hosted VMs are laid out (and moved) by their host, never dragged out.
        draggable: false,
        width: width - VM_INSET * 2,
        height: VM_HEIGHT,
        position: {
          x: VM_INSET,
          y: HOST_HEADER + index * (VM_HEIGHT + VM_GAP),
        },
        data: { device: child, variant: 'vm' },
      })
    })
  }

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
  () => {
    const rackKey = store.racks
      .map((rack) => `${rack.id}:${rack.name}:${rack.location ?? ''}:${rack.totalU}`)
      .join('|')
    const deviceKey = store.devices
      .map(
        (device) =>
          `${device.id}:${device.updatedAt}:${device.kind}:${device.hostId ?? ''}:${device.rackId ?? ''}`,
      )
      .join('|')
    return `${rackKey}#${deviceKey}`
  },
  () => {
    void loadGraph()
  },
)

async function saveLayout(): Promise<void> {
  saving.value = true
  try {
    // Racks and rack-less devices are roots; devices inside racks and hosted
    // VMs are positioned by their parent, so only roots persist.
    const deviceLayout: LayoutNode[] = []
    const rackLayout: RackLayoutNode[] = []

    for (const node of nodes.value) {
      if (node.parentNode) continue
      if (node.type === 'rack') {
        rackLayout.push({
          rackId: Number(node.id.replace('rack-', '')),
          x: node.position.x,
          y: node.position.y,
        })
      } else {
        deviceLayout.push({
          deviceId: Number(node.id),
          x: node.position.x,
          y: node.position.y,
        })
      }
    }

    await api.saveLayout('default', deviceLayout)
    await api.saveRackLayout('default', rackLayout)
    store.notify('拓扑布局已保存')
  } catch (caught) {
    store.notify(caught instanceof Error ? caught.message : '保存布局失败')
  } finally {
    saving.value = false
  }
}

function openLinkModal(prefill?: { sourceId?: string; targetId?: string }): void {
  linkForm.srcDeviceId = prefill?.sourceId ?? ''
  linkForm.srcPort = ''
  linkForm.dstDeviceId = prefill?.targetId ?? ''
  linkForm.dstPort = ''
  linkForm.linkType = 'ethernet'
  linkForm.speed = '1G'
  linkForm.note = ''
  linkError.value = ''
  linkOpen.value = true
}

/**
 * Dragging between connection points must not create an edge silently: open the
 * link dialog with both ends prefilled instead.
 *
 * Handles are named `out` (bottom → source) and `in` (top → target). In loose
 * connection mode Vue Flow normalises the connection by handle type, so the
 * node that owns the `out` handle is always the source.
 */
function onConnect(connection: Connection): void {
  if (connection.source === connection.target) {
    store.notify('不能连接同一台设备')
    return
  }

  let sourceId = connection.source
  let targetId = connection.target

  const sourceHandle = connection.sourceHandle ?? ''
  const targetHandle = connection.targetHandle ?? ''
  if (sourceHandle === 'in' && targetHandle === 'out') {
    sourceId = connection.target
    targetId = connection.source
  } else if (!sourceHandle && !targetHandle) {
    // Defensive fallback for unnamed handles: the upper node becomes the source.
    let sourceY = 0
    let targetY = 0
    for (const node of nodes.value) {
      if (node.id === connection.source) sourceY = node.position.y
      if (node.id === connection.target) targetY = node.position.y
    }
    if (sourceY > targetY) {
      sourceId = connection.target
      targetId = connection.source
    }
  }

  openLinkModal({ sourceId, targetId })
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

function closeMenu(): void {
  menuOpen.value = false
  menuDevice.value = null
}

function pointerPosition(event: NodeMouseEvent['event']): { x: number; y: number } {
  if ('clientX' in event) return { x: event.clientX, y: event.clientY }
  const touch = event.touches[0] ?? event.changedTouches[0]
  return { x: touch?.clientX ?? 0, y: touch?.clientY ?? 0 }
}

function onNodeContextMenu({ event, node }: NodeMouseEvent): void {
  event.preventDefault()
  event.stopPropagation()

  const device = store.deviceById.get(Number(node.id))
  if (!device) return

  const point = pointerPosition(event)
  menuDevice.value = device
  menuPosition.x = point.x
  menuPosition.y = point.y
  menuOpen.value = true
}

function openDeviceForm(device: Device, tab: 'basic' | 'credentials'): void {
  closeMenu()
  deviceFormDevice.value = device
  deviceFormTab.value = tab
  deviceFormOpen.value = true
}

function addLinkFromMenu(): void {
  const device = menuDevice.value
  closeMenu()
  if (!device) return
  openLinkModal({ sourceId: String(device.id) })
}

async function connectFromMenu(
  type: ConnectionTypeSummary,
  action: CredentialAction,
): Promise<void> {
  const device = menuDevice.value
  closeMenu()
  if (!device) return
  await connect(device.id, type, action)
}

function editMenuDevice(): void {
  if (menuDevice.value) openDeviceForm(menuDevice.value, 'basic')
}

function manageMenuCredentials(): void {
  if (menuDevice.value) openDeviceForm(menuDevice.value, 'credentials')
}

async function removeDeviceFromMenu(): Promise<void> {
  const device = menuDevice.value
  closeMenu()
  if (!device) return
  if (!window.confirm(`确定删除设备「${device.name}」吗？相关凭据与连接会一并删除。`)) {
    return
  }
  try {
    await store.deleteDevice(device.id)
    store.notify('设备已删除')
  } catch (caught) {
    store.notify(caught instanceof Error ? caught.message : '删除失败')
  }
}
</script>

<template>
  <div class="space-y-4">
    <div class="flex items-end justify-between">
      <div>
        <h2 class="text-base font-semibold tracking-tight text-slate-100">拓扑</h2>
        <p class="mt-1 text-xs text-slate-500">
          {{ store.racks.length }} 个机柜 · {{ store.devices.length }} 个节点 ·
          {{ store.links.length }} 条连接 · 拖拽连接点创建连接，右键设备查看更多操作
        </p>
      </div>
      <div class="flex gap-2">
        <button class="btn btn-ghost" @click="saveLayout" :disabled="saving">
          <Save class="h-3.5 w-3.5" />
          {{ saving ? '保存中…' : '保存布局' }}
        </button>
        <button class="btn btn-primary" @click="openLinkModal()">
          <Plus class="h-3.5 w-3.5" />
          添加连接
        </button>
      </div>
    </div>

    <div v-if="store.devices.length || store.racks.length" class="grid grid-cols-4 gap-4">
      <section class="card col-span-3 overflow-hidden">
        <div class="h-[calc(100vh-13rem)] min-h-[30rem] w-full">
          <VueFlow
            v-model:nodes="nodes"
            v-model:edges="edges"
            :node-types="nodeTypes"
            :fit-view-on-init="true"
            :min-zoom="0.25"
            :max-zoom="1.6"
            :connection-radius="30"
            :delete-key-code="null"
            :default-edge-options="{ type: 'smoothstep' }"
            @connect="onConnect"
            @node-context-menu="onNodeContextMenu"
            @pane-click="closeMenu"
            @node-drag-start="closeMenu"
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
          <button class="btn btn-ghost mt-1" @click="openLinkModal()">添加第一条</button>
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
      <p class="text-sm font-medium text-slate-300">还没有可展示的机柜或设备</p>
      <p class="max-w-sm text-xs leading-5 text-slate-500">
        先在「机柜」中创建机柜、在「设备」中添加设备，拓扑图会自动生成。
      </p>
      <button class="btn btn-primary" @click="router.push('/devices')">
        前往设备页面
      </button>
    </div>

    <TopologyNodeMenu
      :open="menuOpen"
      :x="menuPosition.x"
      :y="menuPosition.y"
      :device="menuDevice"
      :types="menuTypes"
      :busy="Boolean(connectingKey)"
      @close="closeMenu"
      @connect="connectFromMenu"
      @add-link="addLinkFromMenu"
      @credentials="manageMenuCredentials"
      @edit="editMenuDevice"
      @remove="removeDeviceFromMenu"
    />

    <DeviceFormModal
      :open="deviceFormOpen"
      :device="deviceFormDevice"
      :initial-tab="deviceFormTab"
      @close="deviceFormOpen = false"
      @saved="deviceFormOpen = false"
    />

    <CredentialChooserModal
      :open="chooserOpen"
      :chooser="chooser"
      :device-name="chooser ? deviceName(chooser.deviceId) : ''"
      @close="closeChooser"
      @select="chooseCredential"
    />

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
