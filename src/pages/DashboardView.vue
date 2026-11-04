<script setup lang="ts">
import {
  ArrowUpRight,
  HardDrive,
  KeyRound,
  Network,
  Server,
} from 'lucide-vue-next'
import { computed } from 'vue'
import { useRouter } from 'vue-router'

import {
  deviceKindColor,
  deviceKindLabel,
  deviceKindMeta,
  deviceStatusLabel,
  formatDate,
} from '@/lib/format'
import type { DeviceKind } from '@/lib/types'
import { useAppStore } from '@/stores/app'

const store = useAppStore()
const router = useRouter()

const totalU = computed(() =>
  store.racks.reduce((sum, rack) => sum + rack.totalU, 0),
)

const usedU = computed(() =>
  store.devices.reduce(
    (sum, device) =>
      sum + (device.rackId && device.rackPosition ? device.rackHeight : 0),
    0,
  ),
)

const occupancy = computed(() =>
  totalU.value > 0 ? Math.round((usedU.value / totalU.value) * 100) : 0,
)

const cards = computed(() => [
  {
    label: '机柜',
    value: store.stats.rackCount,
    hint: `${totalU.value}U 总容量`,
    icon: Server,
    to: '/racks',
    accent: 'text-sky-300 bg-sky-500/10 ring-sky-500/20',
  },
  {
    label: '设备',
    value: store.stats.deviceCount,
    hint: `${occupancy.value}% U 位占用`,
    icon: HardDrive,
    to: '/devices',
    accent: 'text-emerald-300 bg-emerald-500/10 ring-emerald-500/20',
  },
  {
    label: '连接',
    value: store.stats.linkCount,
    hint: '网线 / 光纤 / 电源',
    icon: Network,
    to: '/topology',
    accent: 'text-violet-300 bg-violet-500/10 ring-violet-500/20',
  },
  {
    label: '凭据',
    value: store.stats.credentialCount,
    hint: '本机加密存储',
    icon: KeyRound,
    to: '/credentials',
    accent: 'text-amber-300 bg-amber-500/10 ring-amber-500/20',
  },
])

const rackRows = computed(() =>
  store.racks
    .map((rack) => {
      const used = store.devices
        .filter((device) => device.rackId === rack.id && device.rackPosition)
        .reduce((sum, device) => sum + device.rackHeight, 0)
      return {
        rack,
        used,
        pct: rack.totalU > 0 ? Math.min(100, Math.round((used / rack.totalU) * 100)) : 0,
      }
    })
    .sort((a, b) => a.rack.name.localeCompare(b.rack.name)),
)

const kindRows = computed(() => {
  const rows = (Object.keys(deviceKindMeta) as DeviceKind[])
    .map((kind) => ({
      kind,
      label: deviceKindMeta[kind].label,
      color: deviceKindColor(kind),
      count: store.devices.filter((device) => device.kind === kind).length,
    }))
    .filter((row) => row.count > 0)
    .sort((a, b) => b.count - a.count)
  return rows
})

const maxKind = computed(() =>
  Math.max(1, ...kindRows.value.map((row) => row.count)),
)

const recentDevices = computed(() =>
  [...store.devices]
    .sort((a, b) => b.createdAt.localeCompare(a.createdAt))
    .slice(0, 6),
)
</script>

<template>
  <div class="space-y-5">
    <div class="flex items-end justify-between">
      <div>
        <h2 class="text-base font-semibold tracking-tight text-slate-100">机房总览</h2>
        <p class="mt-1 text-xs text-slate-500">
          本机数据 · 最后同步 {{ formatDate(new Date().toISOString()) }}
        </p>
      </div>
      <div class="flex gap-2">
        <button class="btn btn-ghost" @click="router.push('/topology')">
          <Network class="h-3.5 w-3.5" />
          查看拓扑
        </button>
        <button class="btn btn-primary" @click="router.push('/devices')">
          <HardDrive class="h-3.5 w-3.5" />
          新建设备
        </button>
      </div>
    </div>

    <div class="grid grid-cols-4 gap-4">
      <button
        v-for="card in cards"
        :key="card.label"
        class="card p-4 text-left transition hover:border-white/20 hover:bg-slate-900"
        @click="router.push(card.to)"
      >
        <div class="flex items-start justify-between">
          <div>
            <p class="text-[11px] tracking-wide text-slate-500">{{ card.label }}</p>
            <p class="mt-1.5 text-2xl font-semibold tracking-tight text-slate-100">
              {{ card.value }}
            </p>
          </div>
          <div
            class="flex h-8 w-8 items-center justify-center rounded-lg ring-1 ring-inset"
            :class="card.accent"
          >
            <component :is="card.icon" class="h-4 w-4" />
          </div>
        </div>
        <p class="mt-3 text-[10px] text-slate-500">{{ card.hint }}</p>
      </button>
    </div>

    <div class="grid grid-cols-5 gap-4">
      <section class="card col-span-3 p-5">
        <div class="flex items-center justify-between">
          <div>
            <h3 class="text-xs font-semibold tracking-wide text-slate-200">机柜容量</h3>
            <p class="mt-0.5 text-[10px] text-slate-500">
              {{ usedU }}U / {{ totalU }}U 已占用
            </p>
          </div>
          <button
            class="flex items-center gap-1 text-[11px] text-emerald-400 hover:text-emerald-300"
            @click="router.push('/racks')"
          >
            全部机柜
            <ArrowUpRight class="h-3 w-3" />
          </button>
        </div>

        <div v-if="rackRows.length" class="mt-4 space-y-3">
          <div v-for="row in rackRows" :key="row.rack.id" class="space-y-1.5">
            <div class="flex items-center justify-between text-[11px]">
              <button
                class="flex items-center gap-2 text-slate-300 hover:text-emerald-300"
                @click="router.push(`/racks/${row.rack.id}`)"
              >
                <span class="font-medium">{{ row.rack.name }}</span>
                <span class="text-slate-600">{{ row.rack.location ?? '未填写位置' }}</span>
              </button>
              <span class="tabular-nums text-slate-500">
                {{ row.used }} / {{ row.rack.totalU }}U
              </span>
            </div>
            <div class="h-1.5 overflow-hidden rounded-full bg-white/5">
              <div
                class="h-full rounded-full transition-all"
                :class="
                  row.pct > 85
                    ? 'bg-rose-400'
                    : row.pct > 60
                      ? 'bg-amber-400'
                      : 'bg-emerald-400'
                "
                :style="{ width: `${row.pct}%` }"
              />
            </div>
          </div>
        </div>

        <div v-else class="mt-6 rounded-lg border border-dashed border-white/10 p-6 text-center">
          <p class="text-xs text-slate-400">还没有机柜</p>
          <button class="btn btn-primary mt-3" @click="router.push('/racks')">
            创建第一个机柜
          </button>
        </div>
      </section>

      <section class="card col-span-2 p-5">
        <h3 class="text-xs font-semibold tracking-wide text-slate-200">设备类型</h3>
        <p class="mt-0.5 text-[10px] text-slate-500">按类型统计当前设备</p>

        <div v-if="kindRows.length" class="mt-4 space-y-3">
          <div v-for="row in kindRows" :key="row.kind">
            <div class="flex items-center justify-between text-[11px]">
              <span class="flex items-center gap-2 text-slate-300">
                <span
                  class="h-2 w-2 rounded-full"
                  :style="{ backgroundColor: row.color }"
                />
                {{ row.label }}
              </span>
              <span class="tabular-nums text-slate-500">{{ row.count }}</span>
            </div>
            <div class="mt-1 h-1 overflow-hidden rounded-full bg-white/5">
              <div
                class="h-full rounded-full"
                :style="{
                  width: `${(row.count / maxKind) * 100}%`,
                  backgroundColor: row.color,
                }"
              />
            </div>
          </div>
        </div>

        <p v-else class="mt-6 text-center text-xs text-slate-500">暂无设备数据</p>
      </section>
    </div>

    <section class="card overflow-hidden">
      <div class="flex items-center justify-between border-b border-white/5 px-5 py-3.5">
        <h3 class="text-xs font-semibold tracking-wide text-slate-200">最近添加</h3>
        <button
          class="text-[11px] text-emerald-400 hover:text-emerald-300"
          @click="router.push('/devices')"
        >
          查看全部
        </button>
      </div>

      <table v-if="recentDevices.length" class="w-full text-left text-xs">
        <thead class="text-[10px] uppercase tracking-wider text-slate-500">
          <tr class="border-b border-white/5">
            <th class="px-5 py-2.5 font-medium">设备</th>
            <th class="px-5 py-2.5 font-medium">类型</th>
            <th class="px-5 py-2.5 font-medium">管理 IP</th>
            <th class="px-5 py-2.5 font-medium">机柜</th>
            <th class="px-5 py-2.5 font-medium">状态</th>
            <th class="px-5 py-2.5 font-medium">添加时间</th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="device in recentDevices"
            :key="device.id"
            class="border-b border-white/[0.03] transition last:border-0 hover:bg-white/[0.02]"
          >
            <td class="px-5 py-3 font-medium text-slate-200">{{ device.name }}</td>
            <td class="px-5 py-3">
              <span class="badge" :class="deviceKindMeta[device.kind]?.badge">
                {{ deviceKindLabel(device.kind) }}
              </span>
            </td>
            <td class="px-5 py-3 font-mono text-[11px] text-slate-400">
              {{ device.ips[0] ?? '—' }}
              <span v-if="device.ips.length > 1" class="ml-1 text-[9px] text-slate-600">
                +{{ device.ips.length - 1 }}
              </span>
            </td>
            <td class="px-5 py-3 text-slate-400">
              {{ device.rackId ? store.rackById.get(device.rackId)?.name : '—' }}
            </td>
            <td class="px-5 py-3 text-slate-400">
              {{ deviceStatusLabel(device.status) }}
            </td>
            <td class="px-5 py-3 text-[11px] text-slate-500">
              {{ formatDate(device.createdAt) }}
            </td>
          </tr>
        </tbody>
      </table>

      <p v-else class="px-5 py-10 text-center text-xs text-slate-500">
        还没有设备，前往「设备」页面添加第一台。
      </p>
    </section>
  </div>
</template>
