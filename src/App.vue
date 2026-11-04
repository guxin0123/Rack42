<script setup lang="ts">
import { onMounted } from 'vue'

import AppShell from '@/components/AppShell.vue'
import LockScreen from '@/components/LockScreen.vue'
import ToastMessage from '@/components/ToastMessage.vue'
import { useAppStore } from '@/stores/app'
import { useThemeStore } from '@/stores/theme'

const store = useAppStore()
useThemeStore()

onMounted(() => {
  void store.bootstrap()
})
</script>

<template>
  <div
    v-if="store.loading"
    class="flex h-full flex-col items-center justify-center gap-4 bg-slate-950 text-slate-400"
  >
    <div
      class="h-10 w-10 animate-spin rounded-full border-2 border-emerald-400/30 border-t-emerald-400"
    />
    <p class="text-sm tracking-widest">正在打开 Rack42…</p>
  </div>

  <div
    v-else-if="store.bootError"
    class="flex h-full flex-col items-center justify-center gap-5 bg-slate-950 px-6 text-center"
  >
    <div
      class="flex h-14 w-14 items-center justify-center rounded-2xl bg-rose-500/10 text-2xl text-rose-300 ring-1 ring-rose-500/30"
    >
      !
    </div>
    <div class="max-w-xl space-y-2">
      <h1 class="text-lg font-semibold text-slate-100">无法启动本地服务</h1>
      <p class="text-sm leading-6 text-slate-400">{{ store.bootError }}</p>
    </div>
    <button
      class="rounded-lg bg-emerald-500 px-4 py-2 text-sm font-medium text-emerald-950 transition hover:bg-emerald-400"
      @click="store.bootstrap()"
    >
      重试
    </button>
  </div>

  <LockScreen v-else-if="!store.initialized || !store.unlocked" />
  <AppShell v-else />
  <ToastMessage />
</template>
