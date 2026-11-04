<script setup lang="ts">
import { computed } from 'vue'

import { deviceKindColor, deviceKindLabel } from '@/lib/format'
import type { Device, Rack } from '@/lib/types'

const props = defineProps<{
  rack: Rack
  devices: Device[]
}>()

const emit = defineEmits<{
  'slot-click': [u: number]
  'device-click': [device: Device]
}>()

const placed = computed(() =>
  props.devices.filter((device) => device.rackPosition !== null && device.rackHeight > 0),
)

const byTop = computed(() => {
  const map = new Map<number, Device>()
  for (const device of placed.value) {
    const top = (device.rackPosition ?? 1) + device.rackHeight - 1
    map.set(top, device)
  }
  return map
})

const rows = computed(() =>
  Array.from({ length: props.rack.totalU }, (_, index) => props.rack.totalU - index),
)

function occupant(u: number): Device | undefined {
  return placed.value.find((device) => {
    const start = device.rackPosition ?? 0
    return u >= start && u < start + device.rackHeight
  })
}
</script>

<template>
  <div
    class="grid gap-1"
    :style="{ gridTemplateRows: `repeat(${rack.totalU}, 26px)` }"
  >
    <template v-for="u in rows" :key="u">
      <button
        v-if="byTop.get(u)"
        type="button"
        class="flex min-h-0 flex-col justify-center overflow-hidden rounded-md border px-2 text-left transition hover:brightness-110"
        :style="{
          gridRow: `span ${byTop.get(u)!.rackHeight}`,
          borderColor: `${deviceKindColor(byTop.get(u)!.kind)}66`,
          backgroundColor: `${deviceKindColor(byTop.get(u)!.kind)}1a`,
        }"
        @click="emit('device-click', byTop.get(u)!)"
      >
        <span class="truncate text-[11px] font-medium text-slate-100">
          {{ byTop.get(u)!.name }}
        </span>
        <span class="truncate text-[10px] text-slate-400">
          {{ deviceKindLabel(byTop.get(u)!.kind) }}
          <template v-if="byTop.get(u)!.rackHeight > 1">
            · U{{ byTop.get(u)!.rackPosition }}+{{ byTop.get(u)!.rackHeight }}
          </template>
        </span>
      </button>

      <button
        v-else-if="!occupant(u)"
        type="button"
        class="group flex items-center justify-between rounded-md border border-dashed border-white/5 px-2 text-[10px] text-slate-600 transition hover:border-emerald-400/30 hover:text-emerald-300"
        @click="emit('slot-click', u)"
      >
        <span>U{{ u }}</span>
        <span class="opacity-0 transition group-hover:opacity-100">+ 放置设备</span>
      </button>
    </template>
  </div>
</template>
