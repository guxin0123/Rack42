<script setup lang="ts">
import { Server } from 'lucide-vue-next'
import { computed } from 'vue'

import type { Rack } from '@/lib/types'

const props = defineProps<{
  data: {
    rack: Rack
    deviceCount?: number
    usedU?: number
  }
}>()

defineOptions({ inheritAttrs: false })

const rack = computed(() => props.data.rack)
const deviceCount = computed(() => props.data.deviceCount ?? 0)
const usedU = computed(() => props.data.usedU ?? 0)
</script>

<template>
  <div
    class="flex h-full w-full flex-col rounded-2xl border border-sky-500/25 bg-slate-950/40 shadow-lg shadow-black/10"
  >
    <div
      class="flex shrink-0 items-start gap-2 rounded-t-2xl border-b border-white/5 bg-sky-500/10 px-3 py-2.5"
    >
      <Server class="mt-0.5 h-3.5 w-3.5 shrink-0 text-sky-300" />
      <div class="min-w-0 flex-1">
        <div class="flex items-center gap-2">
          <span class="truncate text-xs font-semibold text-slate-100">
            {{ rack.name }}
          </span>
          <span class="shrink-0 rounded bg-white/5 px-1.5 py-0.5 text-[9px] text-slate-400">
            {{ rack.totalU }}U
          </span>
          <span class="ml-auto shrink-0 text-[9px] text-slate-500">
            {{ deviceCount }} 台设备
          </span>
        </div>
        <p class="mt-0.5 truncate text-[10px] text-slate-500">
          {{ rack.location ?? '未填写位置' }} · 已用 {{ usedU }}/{{ rack.totalU }}U
        </p>
      </div>
    </div>

    <div
      v-if="!deviceCount"
      class="flex flex-1 items-center justify-center text-[10px] text-slate-600"
    >
      空机柜
    </div>
    <div v-else class="min-h-0 flex-1" />
  </div>
</template>
