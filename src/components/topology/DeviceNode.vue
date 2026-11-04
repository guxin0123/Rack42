<script setup lang="ts">
import { Handle, Position } from '@vue-flow/core'
import { computed } from 'vue'

import {
  deviceKindColor,
  deviceKindLabel,
  deviceStatusMeta,
  deviceStatusLabel,
} from '@/lib/format'
import type { Device } from '@/lib/types'

const props = defineProps<{
  data: { device: Device }
}>()

const device = computed(() => props.data.device)
const color = computed(() => deviceKindColor(device.value.kind))
</script>

<template>
  <div
    class="w-52 rounded-xl border bg-slate-950/95 px-3 py-2.5 shadow-xl shadow-black/20"
    :style="{ borderColor: `${color}66` }"
  >
    <Handle
      type="target"
      :position="Position.Top"
      class="!h-2 !w-2 !border-0 !bg-slate-600"
    />

    <div class="flex items-center gap-2">
      <span
        class="h-1.5 w-1.5 rounded-full"
        :class="deviceStatusMeta[device.status]?.dot ?? 'bg-slate-500'"
      />
      <span class="truncate text-xs font-semibold text-slate-100">
        {{ device.name }}
      </span>
      <span
        class="ml-auto shrink-0 rounded px-1.5 py-0.5 text-[9px]"
        :style="{ color, backgroundColor: `${color}1f` }"
      >
        {{ deviceKindLabel(device.kind) }}
      </span>
    </div>

    <p class="mt-1 truncate font-mono text-[10px] text-slate-400">
      {{ device.ips[0] ?? '—' }}
    </p>
    <p class="mt-0.5 truncate text-[9px] text-slate-600">
      {{ deviceStatusLabel(device.status) }}
      <template v-if="device.model"> · {{ device.model }}</template>
    </p>

    <Handle
      type="source"
      :position="Position.Bottom"
      class="!h-2 !w-2 !border-0 !bg-slate-600"
    />
  </div>
</template>
