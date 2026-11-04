<script setup lang="ts">
import { CheckCircle2 } from 'lucide-vue-next'

import { useAppStore } from '@/stores/app'

const store = useAppStore()
</script>

<template>
  <Transition
    enter-active-class="transition duration-200 ease-out"
    enter-from-class="translate-y-3 opacity-0"
    enter-to-class="translate-y-0 opacity-100"
    leave-active-class="transition duration-150 ease-in"
    leave-from-class="translate-y-0 opacity-100"
    leave-to-class="translate-y-3 opacity-0"
  >
    <div
      v-if="store.notice"
      class="fixed bottom-6 right-6 z-[100] flex max-w-sm items-start gap-3 rounded-xl border border-white/10 bg-slate-900/95 px-4 py-3 text-sm text-slate-200 shadow-2xl backdrop-blur"
    >
      <CheckCircle2 class="mt-0.5 h-4 w-4 shrink-0 text-emerald-400" />
      <span>{{ store.notice }}</span>
    </div>
  </Transition>
</template>
