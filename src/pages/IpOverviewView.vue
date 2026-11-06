<script setup lang="ts">
import { Globe, Loader2, RefreshCw, Search } from 'lucide-vue-next'
import { computed, onMounted, ref } from 'vue'

import { api } from '@/lib/ipc'
import type { IpOverviewEntry } from '@/lib/types'

const entries = ref<IpOverviewEntry[]>([])
const loading = ref(false)
const error = ref('')
const search = ref('')

async function load(): Promise<void> {
  loading.value = true
  error.value = ''
  try {
    entries.value = await api.listIpOverview()
  } catch (caught) {
    error.value = caught instanceof Error ? caught.message : '加载 IP 总览失败'
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  void load()
})

const filtered = computed(() => {
  const needle = search.value.trim().toLowerCase()
  if (!needle) return entries.value
  return entries.value.filter((entry) =>
    [entry.ip, entry.deviceName, entry.credentialLabel ?? '']
      .some((value) => value.toLowerCase().includes(needle)),
  )
})

const groups = computed(() => {
  const map = new Map<string, IpOverviewEntry[]>()
  for (const entry of filtered.value) {
    const list = map.get(entry.subnet) ?? []
    list.push(entry)
    map.set(entry.subnet, list)
  }
  return [...map.entries()].map(([subnet, items]) => ({ subnet, items }))
})

const uniqueIpCount = computed(() => new Set(entries.value.map((entry) => entry.ip)).size)

const subnetCount = computed(
  () => new Set(entries.value.map((entry) => entry.subnet)).size,
)

const deviceCount = computed(
  () => new Set(entries.value.map((entry) => entry.deviceId)).size,
)

const credentialAddressCount = computed(
  () => entries.value.filter((entry) => entry.source === 'credential').length,
)
</script>

<template>
  <div class="space-y-5">
    <div class="flex flex-wrap items-end justify-between gap-3">
      <div>
        <h2 class="text-base font-semibold tracking-tight text-slate-100">IP 总览</h2>
        <p class="mt-1 text-xs text-slate-500">
          {{ uniqueIpCount }} 个 IP · {{ subnetCount }} 个网段 · {{ deviceCount }} 台设备 ·
          {{ credentialAddressCount }} 条凭据地址
        </p>
      </div>
      <button class="btn btn-ghost" :disabled="loading" @click="load">
        <RefreshCw class="h-3.5 w-3.5" :class="loading ? 'animate-spin' : ''" />
        刷新
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
          placeholder="按 IP、设备名称或凭据标签搜索"
        />
      </div>
    </div>

    <p
      v-if="error"
      class="rounded-lg border border-rose-500/20 bg-rose-500/10 px-3 py-2 text-xs text-rose-300"
    >
      {{ error }}
    </p>

    <div
      v-if="loading && !entries.length"
      class="card flex items-center justify-center gap-2 py-16 text-xs text-slate-500"
    >
      <Loader2 class="h-4 w-4 animate-spin" />
      正在统计 IP 地址…
    </div>

    <template v-else-if="groups.length">
      <section v-for="group in groups" :key="group.subnet" class="card overflow-hidden">
        <div class="flex items-center justify-between border-b border-white/5 px-5 py-3">
          <div class="flex items-center gap-2">
            <Globe class="h-3.5 w-3.5 text-sky-300" />
            <span class="font-mono text-xs font-semibold text-slate-200">
              {{ group.subnet }}
            </span>
          </div>
          <span class="text-[10px] text-slate-500">{{ group.items.length }} 条记录</span>
        </div>

        <table class="w-full text-left text-xs">
          <thead class="text-[10px] uppercase tracking-wider text-slate-500">
            <tr class="border-b border-white/5">
              <th class="px-5 py-2.5 font-medium">IP</th>
              <th class="px-5 py-2.5 font-medium">设备名称</th>
              <th class="px-5 py-2.5 font-medium">来源</th>
              <th class="px-5 py-2.5 font-medium">凭据</th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="(entry, index) in group.items"
              :key="`${entry.ip}-${entry.deviceId}-${entry.source}-${entry.credentialId ?? index}`"
              class="border-b border-white/[0.03] last:border-0 hover:bg-white/[0.02]"
            >
              <td class="px-5 py-3 font-mono text-[11px] text-slate-200">
                {{ entry.ip }}
              </td>
              <td class="px-5 py-3 text-slate-300">{{ entry.deviceName }}</td>
              <td class="px-5 py-3">
                <span
                  class="badge"
                  :class="
                    entry.source === 'device'
                      ? 'bg-sky-500/15 text-sky-300 ring-sky-500/30'
                      : 'bg-amber-500/15 text-amber-300 ring-amber-500/30'
                  "
                >
                  {{ entry.source === 'device' ? '设备 IP' : '凭据地址' }}
                </span>
              </td>
              <td class="px-5 py-3 text-slate-400">
                {{ entry.credentialLabel ?? '—' }}
              </td>
            </tr>
          </tbody>
        </table>
      </section>
    </template>

    <div
      v-else-if="entries.length"
      class="card flex flex-col items-center justify-center gap-2 px-6 py-16 text-center"
    >
      <p class="text-sm font-medium text-slate-300">没有匹配的 IP</p>
      <p class="text-xs text-slate-500">试试调整搜索关键字。</p>
    </div>

    <div
      v-else
      class="card flex flex-col items-center justify-center gap-3 px-6 py-16 text-center"
    >
      <Globe class="h-6 w-6 text-slate-600" />
      <p class="text-sm font-medium text-slate-300">还没有可统计的 IP</p>
      <p class="max-w-md text-xs leading-5 text-slate-500">
        在设备中填写管理 IP，或在凭据中填写 IP / URL 地址后，这里会按网段汇总（IPv4 按 /24、IPv6 按 /64 分组）。
      </p>
    </div>
  </div>
</template>
