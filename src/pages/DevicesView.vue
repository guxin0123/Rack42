<script setup lang="ts">
import {
  ChevronDown,
  ChevronRight,
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

import CredentialChooserModal from '@/components/device/CredentialChooserModal.vue'
import DeviceFormModal from '@/components/device/DeviceFormModal.vue'
import { actionLabels, useDeviceConnect } from '@/composables/useDeviceConnect'
import {
  deviceKindLabel,
  deviceKindMeta,
  deviceStatusLabel,
  deviceStatusMeta,
} from '@/lib/format'
import type { Device, DeviceKind } from '@/lib/types'
import { useAppStore } from '@/stores/app'

const store = useAppStore()
const route = useRoute()
const {
  chooserOpen,
  chooser,
  connectionTypes,
  isConnecting,
  connect,
  connectFirst,
  chooseCredential,
  closeChooser,
} = useDeviceConnect()

const search = ref('')
const kindFilter = ref<'all' | DeviceKind>('all')
const rackFilter = ref<'all' | 'unassigned' | number>('all')
const formOpen = ref(false)
const editing = ref<Device | null>(null)

function deviceNameOf(deviceId: number): string {
  return store.deviceById.get(deviceId)?.name ?? ''
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

interface DeviceRow {
  device: Device
  depth: number
  childCount: number
}

const collapsedHosts = ref<Set<number>>(new Set())

const filtersActive = computed(
  () =>
    search.value.trim().length > 0 ||
    kindFilter.value !== 'all' ||
    rackFilter.value !== 'all',
)

const rows = computed<DeviceRow[]>(() => {
  const matchedIds = new Set(filtered.value.map((device) => device.id))

  const childrenByHost = new Map<number, Device[]>()
  const hostedIds = new Set<number>()
  for (const device of store.devices) {
    if (device.kind !== 'vm' || !device.hostId) continue
    const host = store.deviceById.get(device.hostId)
    if (!host || host.id === device.id) continue
    const list = childrenByHost.get(host.id) ?? []
    list.push(device)
    childrenByHost.set(host.id, list)
    hostedIds.add(device.id)
  }
  for (const list of childrenByHost.values()) {
    list.sort((a, b) => a.name.localeCompare(b.name))
  }

  const topLevel = store.devices
    .filter((device) => !hostedIds.has(device.id))
    .sort((a, b) => a.name.localeCompare(b.name))

  const result: DeviceRow[] = []

  for (const device of topLevel) {
    const children = childrenByHost.get(device.id) ?? []
    const hostMatches = matchedIds.has(device.id)
    const matchedChildren = children.filter((child) => matchedIds.has(child.id))
    // A matching VM must keep its host visible, even when the host itself does
    // not match the current filters.
    if (!hostMatches && !matchedChildren.length) continue

    result.push({
      device,
      depth: 0,
      childCount: children.length,
    })

    const expanded = filtersActive.value || !collapsedHosts.value.has(device.id)
    if (!expanded) continue

    for (const child of hostMatches ? children : matchedChildren) {
      result.push({ device: child, depth: 1, childCount: 0 })
    }
  }

  return result
})

const anyCollapsible = computed(() => rows.value.some((row) => row.childCount > 0))

const allCollapsed = computed(() => {
  const hosts = rows.value.filter((row) => row.childCount > 0)
  return (
    hosts.length > 0 && hosts.every((row) => collapsedHosts.value.has(row.device.id))
  )
})

function isCollapsed(deviceId: number): boolean {
  return !filtersActive.value && collapsedHosts.value.has(deviceId)
}

function toggleHost(deviceId: number): void {
  const next = new Set(collapsedHosts.value)
  if (next.has(deviceId)) next.delete(deviceId)
  else next.add(deviceId)
  collapsedHosts.value = next
}

function toggleAll(): void {
  if (allCollapsed.value) {
    collapsedHosts.value = new Set()
    return
  }
  collapsedHosts.value = new Set(
    rows.value.filter((row) => row.childCount > 0).map((row) => row.device.id),
  )
}

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
          共 {{ filtered.length }} 台 / {{ store.devices.length }} 台 ·
          宿主机与虚拟机按树形展示，双击设备可直接连接第一条凭据
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

      <button
        class="btn btn-ghost"
        type="button"
        :disabled="!anyCollapsible || filtersActive"
        @click="toggleAll"
      >
        {{ allCollapsed ? '展开全部' : '收起全部' }}
      </button>
    </div>

    <div class="card overflow-x-auto">
      <table v-if="rows.length" class="w-full text-left text-xs">
        <thead class="text-[10px] uppercase tracking-wider text-slate-500">
          <tr class="border-b border-white/5">
            <th class="px-5 py-3 font-medium">设备 / 虚拟机</th>
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
            v-for="row in rows"
            :key="row.device.id"
            class="select-none border-b border-white/[0.03] transition last:border-0 hover:bg-white/[0.02]"
            @dblclick="connectFirst(row.device)"
          >
            <td
              class="px-5 py-3"
              :style="row.depth ? { paddingLeft: '2.75rem' } : undefined"
            >
              <div class="flex items-start gap-1.5">
                <button
                  v-if="row.childCount"
                  type="button"
                  class="mt-0.5 shrink-0 rounded p-0.5 text-slate-500 transition hover:bg-white/5 hover:text-slate-200 disabled:opacity-40"
                  :disabled="filtersActive"
                  :title="
                    filtersActive
                      ? '筛选状态下自动展开'
                      : isCollapsed(row.device.id)
                        ? '展开虚拟机'
                        : '收起虚拟机'
                  "
                  @click.stop="toggleHost(row.device.id)"
                  @dblclick.stop
                >
                  <ChevronRight v-if="isCollapsed(row.device.id)" class="h-3.5 w-3.5" />
                  <ChevronDown v-else class="h-3.5 w-3.5" />
                </button>
                <span v-else-if="row.depth" class="mt-0.5 shrink-0 text-slate-700">└</span>

                <div class="min-w-0">
                  <div class="flex items-center gap-1.5">
                    <p class="truncate font-medium text-slate-200">
                      {{ row.device.name }}
                    </p>
                    <span
                      v-if="row.childCount"
                      class="shrink-0 rounded bg-white/5 px-1.5 py-0.5 text-[9px] text-slate-500"
                    >
                      {{ row.childCount }} 台虚拟机
                    </span>
                  </div>
                  <p class="mt-0.5 truncate font-mono text-[10px] text-slate-500">
                    <template v-if="row.depth && row.device.hostId">
                      宿主：{{ deviceNameOf(row.device.hostId) }}
                    </template>
                    <template v-else>{{ row.device.serial ?? '—' }}</template>
                  </p>
                </div>
              </div>
            </td>
            <td class="px-5 py-3">
              <span class="badge" :class="deviceKindMeta[row.device.kind]?.badge">
                {{ deviceKindLabel(row.device.kind) }}
              </span>
            </td>
            <td class="px-5 py-3 font-mono text-[11px] text-slate-400">
              <span>{{ row.device.ips[0] ?? '—' }}</span>
              <span
                v-if="row.device.ips.length > 1"
                class="ml-1.5 rounded bg-white/5 px-1 py-0.5 text-[9px] text-slate-500"
                :title="row.device.ips.join('\n')"
              >
                +{{ row.device.ips.length - 1 }}
              </span>
            </td>
            <td class="px-5 py-3 text-slate-400">
              <template v-if="row.device.rackId">
                <span class="text-slate-300">
                  {{ store.rackById.get(row.device.rackId)?.name ?? '—' }}
                </span>
                <span
                  v-if="row.device.rackPosition"
                  class="ml-1.5 font-mono text-[10px] text-slate-500"
                >
                  U{{ row.device.rackPosition }}
                </span>
              </template>
              <span v-else class="text-slate-600">未分配</span>
            </td>
            <td class="px-5 py-3">
              <span class="flex items-center gap-1.5 text-slate-400">
                <span
                  class="h-1.5 w-1.5 rounded-full"
                  :class="deviceStatusMeta[row.device.status]?.dot"
                />
                {{ deviceStatusLabel(row.device.status) }}
              </span>
            </td>
            <td class="px-5 py-3 text-slate-400">
              {{ [row.device.vendor, row.device.model].filter(Boolean).join(' ') || '—' }}
            </td>
            <td class="px-5 py-3" @dblclick.stop>
              <div class="flex justify-end gap-1">
                <template
                  v-for="type in connectionTypes(row.device.id)"
                  :key="type.typeId"
                >
                  <button
                    v-for="action in type.actions"
                    :key="`${type.typeId}-${action}`"
                    class="rounded-md p-1.5 text-emerald-300 transition hover:bg-emerald-500/10 disabled:opacity-40"
                    :title="`${type.name}：${actionLabels[action] ?? action}（${type.credentialCount} 条凭据${type.credentialCount > 1 ? '，点击选择' : ''}）`"
                    :disabled="isConnecting(row.device.id, type.typeId, action)"
                    @click="connect(row.device.id, type, action)"
                  >
                    <Loader2
                      v-if="isConnecting(row.device.id, type.typeId, action)"
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
                  @click="editDevice(row.device)"
                >
                  <Pencil class="h-3.5 w-3.5" />
                </button>
                <button
                  class="rounded-md p-1.5 text-slate-500 transition hover:bg-rose-500/10 hover:text-rose-300"
                  title="删除"
                  @click="removeDevice(row.device)"
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

    <CredentialChooserModal
      :open="chooserOpen"
      :chooser="chooser"
      :device-name="chooser ? deviceNameOf(chooser.deviceId) : ''"
      @close="closeChooser"
      @select="chooseCredential"
    />
  </div>
</template>
