<script setup lang="ts">
import {
  FolderUp,
  Globe,
  Loader2,
  Monitor,
  Pencil,
  Plus,
  Search,
  Terminal,
  Trash2,
} from 'lucide-vue-next'
import { computed, ref, watch } from 'vue'
import { useRoute } from 'vue-router'

import DeviceFormModal from '@/components/device/DeviceFormModal.vue'
import Modal from '@/components/ui/Modal.vue'
import {
  deviceKindLabel,
  deviceKindMeta,
  deviceStatusLabel,
  deviceStatusMeta,
} from '@/lib/format'
import { api } from '@/lib/ipc'
import type {
  ConnectionCredentialSummary,
  ConnectionTypeSummary,
  CredentialAction,
  Device,
  DeviceKind,
} from '@/lib/types'
import { useAppStore } from '@/stores/app'

const store = useAppStore()
const route = useRoute()

const search = ref('')
const kindFilter = ref<'all' | DeviceKind>('all')
const rackFilter = ref<'all' | 'unassigned' | number>('all')
const formOpen = ref(false)
const editing = ref<Device | null>(null)
const connectingKey = ref('')
const chooserOpen = ref(false)
const chooser = ref<{
  type: ConnectionTypeSummary
  action: CredentialAction
} | null>(null)

function connectionActions(type: ConnectionTypeSummary): CredentialAction[] {
  return type.actions
}

const actionLabels: Record<string, string> = {
  rdp: '远程桌面',
  web: '打开 Web',
  ssh: 'SSH 终端',
  sftp: 'SFTP 文件传输',
}

const actionStarted: Record<string, string> = {
  rdp: '已启动远程桌面',
  web: '已在浏览器中打开',
  ssh: '已启动 SSH 终端',
  sftp: '已启动 SFTP 客户端',
}

function connectionTypes(deviceId: number): ConnectionTypeSummary[] {
  const types =
    store.deviceConnections.find((item) => item.deviceId === deviceId)?.types ?? []
  return types.filter((type) => connectionActions(type).length > 0)
}

async function connect(type: ConnectionTypeSummary, action: CredentialAction): Promise<void> {
  if (type.credentials.length > 1) {
    chooser.value = { type, action }
    chooserOpen.value = true
    return
  }

  const credentialId = type.credentials[0]?.id ?? type.defaultCredentialId
  await startConnection(credentialId, action, `${type.typeId}-${action}`)
}

async function chooseCredential(credential: ConnectionCredentialSummary): Promise<void> {
  const context = chooser.value
  chooserOpen.value = false
  chooser.value = null
  if (!context) return
  await startConnection(
    credential.id,
    context.action,
    `${context.type.typeId}-${context.action}`,
  )
}

async function startConnection(
  credentialId: number,
  action: CredentialAction,
  key: string,
): Promise<void> {
  if (connectingKey.value) return
  connectingKey.value = key

  try {
    await api.connectCredential(credentialId, action)
    store.notify(actionStarted[action] ?? '已启动连接')
  } catch (caught) {
    store.notify(caught instanceof Error ? caught.message : '连接失败')
  } finally {
    connectingKey.value = ''
  }
}

const kindOptions = Object.entries(deviceKindMeta).map(([value, meta]) => ({
  value: value as DeviceKind,
  label: meta.label,
}))

function syncQuery(): void {
  const q = route.query.q
  if (typeof q === 'string' && q) search.value = q
}

watch(() => route.query.q, syncQuery, { immediate: true })

const filtered = computed(() => {
  const needle = search.value.trim().toLowerCase()
  return store.devices
    .filter((device) => {
      if (kindFilter.value !== 'all' && device.kind !== kindFilter.value) return false
      if (rackFilter.value === 'unassigned' && device.rackId !== null) return false
      if (
        typeof rackFilter.value === 'number' &&
        device.rackId !== rackFilter.value
      ) {
        return false
      }
      if (!needle) return true
      return [
        device.name,
        ...device.ips,
        device.vendor,
        device.model,
        device.serial,
        device.os,
      ]
        .filter(Boolean)
        .some((value) => (value as string).toLowerCase().includes(needle))
    })
    .sort((a, b) => a.name.localeCompare(b.name))
})

function createDevice(): void {
  editing.value = null
  formOpen.value = true
}

function editDevice(device: Device): void {
  editing.value = device
  formOpen.value = true
}

async function removeDevice(device: Device): Promise<void> {
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
  <div class="space-y-5">
    <div class="flex items-end justify-between">
      <div>
        <h2 class="text-base font-semibold tracking-tight text-slate-100">设备</h2>
        <p class="mt-1 text-xs text-slate-500">
          共 {{ filtered.length }} 台 / {{ store.devices.length }} 台
        </p>
      </div>
      <button class="btn btn-primary" @click="createDevice">
        <Plus class="h-3.5 w-3.5" />
        新增设备
      </button>
    </div>

    <div class="card flex flex-wrap items-center gap-3 p-3">
      <div class="relative min-w-64 flex-1">
        <Search
          class="pointer-events-none absolute left-3 top-1/2 h-3.5 w-3.5 -translate-y-1/2 text-slate-500"
        />
        <input
          v-model="search"
          class="input pl-9"
          placeholder="按名称、IP、厂商、型号、序列号搜索"
        />
      </div>

      <select v-model="kindFilter" class="input w-36">
        <option value="all">全部类型</option>
        <option v-for="option in kindOptions" :key="option.value" :value="option.value">
          {{ option.label }}
        </option>
      </select>

      <select v-model="rackFilter" class="input w-40">
        <option value="all">全部机柜</option>
        <option value="unassigned">未分配机柜</option>
        <option v-for="rack in store.racks" :key="rack.id" :value="rack.id">
          {{ rack.name }}
        </option>
      </select>
    </div>

    <div class="card overflow-x-auto">
      <table v-if="filtered.length" class="w-full text-left text-xs">
        <thead class="text-[10px] uppercase tracking-wider text-slate-500">
          <tr class="border-b border-white/5">
            <th class="px-5 py-3 font-medium">设备</th>
            <th class="px-5 py-3 font-medium">类型</th>
            <th class="px-5 py-3 font-medium">管理 IP</th>
            <th class="px-5 py-3 font-medium">机柜 / U 位</th>
            <th class="px-5 py-3 font-medium">状态</th>
            <th class="px-5 py-3 font-medium">厂商 / 型号</th>
            <th class="px-5 py-3 text-right font-medium">操作</th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="device in filtered"
            :key="device.id"
            class="border-b border-white/[0.03] transition last:border-0 hover:bg-white/[0.02]"
          >
            <td class="px-5 py-3">
              <p class="font-medium text-slate-200">{{ device.name }}</p>
              <p class="mt-0.5 font-mono text-[10px] text-slate-500">
                {{ device.serial ?? '—' }}
              </p>
            </td>
            <td class="px-5 py-3">
              <span class="badge" :class="deviceKindMeta[device.kind]?.badge">
                {{ deviceKindLabel(device.kind) }}
              </span>
            </td>
            <td class="px-5 py-3 font-mono text-[11px] text-slate-400">
              <span>{{ device.ips[0] ?? '—' }}</span>
              <span
                v-if="device.ips.length > 1"
                class="ml-1.5 rounded bg-white/5 px-1 py-0.5 text-[9px] text-slate-500"
                :title="device.ips.join('\n')"
              >
                +{{ device.ips.length - 1 }}
              </span>
            </td>
            <td class="px-5 py-3 text-slate-400">
              <template v-if="device.rackId">
                <span class="text-slate-300">
                  {{ store.rackById.get(device.rackId)?.name ?? '—' }}
                </span>
                <span v-if="device.rackPosition" class="ml-1.5 font-mono text-[10px] text-slate-500">
                  U{{ device.rackPosition }}
                </span>
              </template>
              <span v-else class="text-slate-600">未分配</span>
            </td>
            <td class="px-5 py-3">
              <span class="flex items-center gap-1.5 text-slate-400">
                <span
                  class="h-1.5 w-1.5 rounded-full"
                  :class="deviceStatusMeta[device.status]?.dot"
                />
                {{ deviceStatusLabel(device.status) }}
              </span>
            </td>
            <td class="px-5 py-3 text-slate-400">
              {{ [device.vendor, device.model].filter(Boolean).join(' ') || '—' }}
            </td>
            <td class="px-5 py-3">
              <div class="flex justify-end gap-1">
                <template
                  v-for="type in connectionTypes(device.id)"
                  :key="type.typeId"
                >
                  <button
                    v-for="action in connectionActions(type)"
                    :key="`${type.typeId}-${action}`"
                    class="rounded-md p-1.5 text-emerald-300 transition hover:bg-emerald-500/10 disabled:opacity-40"
                    :title="`${type.name}：${actionLabels[action] ?? action}（${type.credentialCount} 条凭据${type.credentialCount > 1 ? '，点击选择' : ''}）`"
                    :disabled="connectingKey === `${type.typeId}-${action}`"
                    @click="connect(type, action)"
                  >
                    <Loader2
                      v-if="connectingKey === `${type.typeId}-${action}`"
                      class="h-3.5 w-3.5 animate-spin"
                    />
                    <Monitor v-else-if="action === 'rdp'" class="h-3.5 w-3.5" />
                    <Globe v-else-if="action === 'web'" class="h-3.5 w-3.5" />
                    <Terminal v-else-if="action === 'ssh'" class="h-3.5 w-3.5" />
                    <FolderUp v-else class="h-3.5 w-3.5" />
                  </button>
                </template>
                <button
                  class="rounded-md p-1.5 text-slate-400 transition hover:bg-white/10 hover:text-slate-200"
                  title="编辑"
                  @click="editDevice(device)"
                >
                  <Pencil class="h-3.5 w-3.5" />
                </button>
                <button
                  class="rounded-md p-1.5 text-slate-500 transition hover:bg-rose-500/10 hover:text-rose-300"
                  title="删除"
                  @click="removeDevice(device)"
                >
                  <Trash2 class="h-3.5 w-3.5" />
                </button>
              </div>
            </td>
          </tr>
        </tbody>
      </table>

      <div
        v-else
        class="flex flex-col items-center justify-center gap-3 px-6 py-16 text-center"
      >
        <p class="text-sm font-medium text-slate-300">
          {{ store.devices.length ? '没有匹配的设备' : '还没有设备' }}
        </p>
        <p class="max-w-sm text-xs leading-5 text-slate-500">
          {{
            store.devices.length
              ? '试试调整搜索关键字或筛选条件。'
              : '添加设备后可以在机柜 U 位视图和拓扑图中查看它。'
          }}
        </p>
        <button
          v-if="!store.devices.length"
          class="btn btn-primary mt-1"
          @click="createDevice"
        >
          <Plus class="h-3.5 w-3.5" />
          新增第一台设备
        </button>
      </div>
    </div>

    <DeviceFormModal
      :open="formOpen"
      :device="editing"
      @close="formOpen = false"
      @saved="formOpen = false"
    />

    <Modal
      :open="chooserOpen"
      title="选择凭据"
      width="max-w-md"
      @close="chooserOpen = false"
    >
      <div v-if="chooser" class="space-y-2">
        <p class="mb-3 text-[11px] text-slate-500">
          {{ chooser.type.name }} · {{ actionLabels[chooser.action] ?? chooser.action }}
        </p>
        <button
          v-for="credential in chooser.type.credentials"
          :key="credential.id"
          class="flex w-full items-center justify-between gap-3 rounded-xl border border-white/5 bg-white/[0.02] px-3 py-2.5 text-left transition hover:border-emerald-400/30 hover:bg-emerald-400/5"
          @click="chooseCredential(credential)"
        >
          <span class="min-w-0">
            <span class="block truncate text-xs font-medium text-slate-200">
              {{ credential.label ?? '未命名' }}
            </span>
            <span class="mt-0.5 block truncate font-mono text-[10px] text-slate-500">
              {{ credential.username || '无用户名' }}
            </span>
          </span>
          <span
            v-if="credential.isDefault"
            class="shrink-0 rounded bg-emerald-500/10 px-1.5 py-0.5 text-[9px] text-emerald-300"
          >
            默认
          </span>
        </button>
      </div>
      <template #footer>
        <button class="btn btn-ghost" @click="chooserOpen = false">取消</button>
      </template>
    </Modal>
  </div>
</template>
