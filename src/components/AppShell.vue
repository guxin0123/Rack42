<script setup lang="ts">
import {
  ChevronRight,
  HardDrive,
  KeyRound,
  LayoutDashboard,
  Lock,
  Network,
  Search,
  Server,
  Settings,
  ShieldCheck,
} from 'lucide-vue-next'
import { computed, ref } from 'vue'
import { RouterLink, RouterView, useRoute, useRouter } from 'vue-router'

import {
  deviceKindLabel,
  deviceStatusLabel,
} from '@/lib/format'
import { useAppStore } from '@/stores/app'

interface SearchHit {
  key: string
  title: string
  subtitle: string
  to: string
  type: '机柜' | '设备'
}

const store = useAppStore()
const route = useRoute()
const router = useRouter()

const query = ref('')
const searchOpen = ref(false)
let blurTimer: ReturnType<typeof setTimeout> | undefined

const nav = [
  { to: '/', label: '总览', icon: LayoutDashboard },
  { to: '/racks', label: '机柜', icon: Server },
  { to: '/devices', label: '设备', icon: HardDrive },
  { to: '/topology', label: '拓扑', icon: Network },
  { to: '/credentials', label: '凭据', icon: KeyRound },
  { to: '/settings', label: '设置', icon: Settings },
]

const pageTitle = computed(() => (route.meta.title as string | undefined) ?? 'Rack42')

const hits = computed<SearchHit[]>(() => {
  const needle = query.value.trim().toLowerCase()
  if (!needle) return []

  const rackHits: SearchHit[] = store.racks
    .filter(
      (rack) =>
        rack.name.toLowerCase().includes(needle) ||
        (rack.location ?? '').toLowerCase().includes(needle),
    )
    .slice(0, 4)
    .map((rack) => ({
      key: `rack-${rack.id}`,
      title: rack.name,
      subtitle: `${rack.location ?? '未填写位置'} · ${rack.totalU}U`,
      to: `/racks/${rack.id}`,
      type: '机柜',
    }))

  const deviceHits: SearchHit[] = store.devices
    .filter((device) =>
      [device.name, ...device.ips, device.vendor, device.model, device.serial]
        .filter(Boolean)
        .some((value) => (value as string).toLowerCase().includes(needle)),
    )
    .slice(0, 6)
    .map((device) => ({
      key: `device-${device.id}`,
      title: device.name,
      subtitle: `${deviceKindLabel(device.kind)} · ${deviceStatusLabel(device.status)} · ${
        device.ips[0] ?? '无 IP'
      }`,
      to: `/devices?q=${encodeURIComponent(device.name)}`,
      type: '设备',
    }))

  return [...rackHits, ...deviceHits].slice(0, 8)
})

function isActive(to: string): boolean {
  if (to === '/') return route.path === '/'
  return route.path === to || route.path.startsWith(`${to}/`)
}

function go(hit: SearchHit): void {
  query.value = ''
  searchOpen.value = false
  void router.push(hit.to)
}

function onSubmitSearch(): void {
  if (hits.value.length > 0) go(hits.value[0])
}

function onBlur(): void {
  blurTimer = setTimeout(() => {
    searchOpen.value = false
  }, 150)
}

function onFocus(): void {
  if (blurTimer) clearTimeout(blurTimer)
  searchOpen.value = true
}

async function lockNow(): Promise<void> {
  try {
    await store.lock()
  } catch (caught) {
    store.notify(caught instanceof Error ? caught.message : '锁定失败')
  }
}
</script>

<template>
  <div class="flex h-full bg-slate-950">
    <aside
      class="flex w-60 shrink-0 flex-col border-r border-white/5 bg-slate-950/80 px-3 py-4"
    >
      <div class="mb-6 flex items-center gap-3 px-2">
        <div
          class="flex h-10 w-10 items-center justify-center rounded-xl border border-emerald-400/30 bg-emerald-400/10 text-base font-bold text-emerald-300"
        >
          42
        </div>
        <div>
          <p class="text-sm font-semibold tracking-tight text-slate-100">Rack42</p>
          <p class="text-[10px] tracking-[0.2em] text-slate-500">RACK CONTROL</p>
        </div>
      </div>

      <nav class="space-y-1">
        <RouterLink
          v-for="item in nav"
          :key="item.to"
          :to="item.to"
          class="group flex items-center gap-3 rounded-lg px-3 py-2 text-sm transition"
          :class="
            isActive(item.to)
              ? 'bg-emerald-400/10 text-emerald-300 ring-1 ring-inset ring-emerald-400/20'
              : 'text-slate-400 hover:bg-white/5 hover:text-slate-200'
          "
        >
          <component :is="item.icon" class="h-4 w-4 shrink-0" />
          <span>{{ item.label }}</span>
          <ChevronRight
            v-if="isActive(item.to)"
            class="ml-auto h-3.5 w-3.5 text-emerald-400/70"
          />
        </RouterLink>
      </nav>

      <div class="mt-auto space-y-3 px-2">
        <div class="rounded-lg border border-white/5 bg-white/[0.02] p-3">
          <div class="flex items-center gap-2 text-[11px] text-slate-400">
            <ShieldCheck class="h-3.5 w-3.5 text-emerald-400" />
            <span>已解锁 · 本机加密</span>
          </div>
          <p class="mt-1.5 break-all text-[10px] leading-4 text-slate-600">
            {{ store.dataDir }}
          </p>
        </div>

        <button
          class="flex w-full items-center justify-center gap-2 rounded-lg border border-white/10 px-3 py-2 text-xs text-slate-400 transition hover:border-white/20 hover:text-slate-200"
          @click="lockNow"
        >
          <Lock class="h-3.5 w-3.5" />
          锁定
        </button>
        <p class="text-center text-[10px] tracking-wide text-slate-700">
          Rack42 v0.1.0
        </p>
      </div>
    </aside>

    <div class="flex min-w-0 flex-1 flex-col">
      <header
        class="flex h-16 shrink-0 items-center justify-between gap-6 border-b border-white/5 bg-slate-950/60 px-6 backdrop-blur"
      >
        <div>
          <h1 class="text-sm font-semibold tracking-wide text-slate-100">
            {{ pageTitle }}
          </h1>
          <p class="mt-0.5 text-[11px] text-slate-500">
            {{ store.stats.rackCount }} 个机柜 · {{ store.stats.deviceCount }} 台设备 ·
            {{ store.stats.linkCount }} 条连接
          </p>
        </div>

        <div class="relative w-full max-w-sm">
          <Search
            class="pointer-events-none absolute left-3 top-1/2 h-3.5 w-3.5 -translate-y-1/2 text-slate-500"
          />
          <input
            v-model="query"
            type="text"
            placeholder="搜索机柜、设备、IP、序列号…"
            class="w-full rounded-lg border border-white/10 bg-slate-900/60 py-2 pl-9 pr-3 text-xs text-slate-200 outline-none transition placeholder:text-slate-600 focus:border-emerald-400/50 focus:ring-2 focus:ring-emerald-400/10"
            @focus="onFocus"
            @blur="onBlur"
            @keydown.enter.prevent="onSubmitSearch"
            @keydown.esc="searchOpen = false"
          />

          <div
            v-if="searchOpen && query.trim()"
            class="absolute left-0 right-0 top-full z-40 mt-2 overflow-hidden rounded-xl border border-white/10 bg-slate-900 shadow-2xl"
          >
            <button
              v-for="hit in hits"
              :key="hit.key"
              class="flex w-full items-center gap-3 px-3 py-2.5 text-left transition hover:bg-white/5"
              @mousedown.prevent="go(hit)"
            >
              <span
                class="rounded bg-white/5 px-1.5 py-0.5 text-[10px] text-slate-500"
              >
                {{ hit.type }}
              </span>
              <span class="min-w-0 flex-1">
                <span class="block truncate text-xs font-medium text-slate-200">
                  {{ hit.title }}
                </span>
                <span class="block truncate text-[10px] text-slate-500">
                  {{ hit.subtitle }}
                </span>
              </span>
            </button>
            <p
              v-if="hits.length === 0"
              class="px-3 py-3 text-center text-[11px] text-slate-500"
            >
              没有匹配的机柜或设备
            </p>
          </div>
        </div>
      </header>

      <main class="min-h-0 flex-1 overflow-y-auto px-6 py-6">
        <RouterView />
      </main>
    </div>
  </div>
</template>
