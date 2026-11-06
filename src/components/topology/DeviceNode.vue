<script setup lang="ts">
import { Handle, Position } from '@vue-flow/core'
import { computed } from 'vue'

import {
  deviceKindColor,
  deviceKindLabel,
  deviceStatusMeta,
  deviceStatusLabel,
} from '@/lib/format'
import { iconComponent } from '@/lib/icons'
import { osFamilyInfo } from '@/lib/os'
import { useTopologyHighlight } from '@/composables/useTopologyHighlight'
import { useAppStore } from '@/stores/app'
import type { Device } from '@/lib/types'

const props = defineProps<{
  data: {
    device: Device
    childCount?: number
    variant?: 'device' | 'host' | 'vm' | 'racked'
  }
}>()

defineOptions({ inheritAttrs: false })

const device = computed(() => props.data.device)
const color = computed(() => deviceKindColor(device.value.kind))
const variant = computed(() => props.data.variant ?? 'device')
const childCount = computed(() => props.data.childCount ?? 0)
const highlighted = useTopologyHighlight()
const isHighlighted = computed(() => highlighted.value.has(device.value.id))

const store = useAppStore()
const osFamily = computed(() => osFamilyInfo(device.value.osFamily, store.osFamilies))
const osIcon = computed(() => (osFamily.value ? iconComponent(osFamily.value.icon) : null))
const osTitle = computed(() =>
  osFamily.value
    ? `${osFamily.value.name}${device.value.os ? ` · ${device.value.os}` : ''}`
    : (device.value.os ?? ''),
)
</script>

<template>
  <!-- Host container: VMs are rendered as child nodes floating above the body. -->
  <div
    v-if="variant === 'host'"
    class="flex h-full w-full flex-col rounded-2xl border bg-slate-950/95 shadow-xl shadow-black/25"
    :class="isHighlighted ? 'ring-2 ring-emerald-400/80' : ''"
    :style="{ borderColor: `${color}80` }"
  >
    <Handle
      id="in"
      type="target"
      :position="Position.Top"
      class="!h-2.5 !w-2.5 !border-0 !bg-slate-500 transition hover:!bg-emerald-400"
    />

    <div
      class="flex shrink-0 items-center gap-2 rounded-t-2xl border-b border-white/5 px-3 py-2.5"
      :style="{ backgroundColor: `${color}14` }"
    >
      <span
        class="h-1.5 w-1.5 rounded-full"
        :class="deviceStatusMeta[device.status]?.dot ?? 'bg-slate-500'"
      />
      <span class="truncate text-xs font-semibold text-slate-100">
        {{ device.name }}
      </span>
      <span
        class="shrink-0 rounded px-1.5 py-0.5 text-[9px]"
        :style="{ color, backgroundColor: `${color}1f` }"
      >
        {{ deviceKindLabel(device.kind) }}
      </span>
      <span v-if="osIcon" class="shrink-0 text-slate-500" :title="osTitle">
        <component :is="osIcon" class="h-3 w-3" />
      </span>
      <span
        class="ml-auto shrink-0 rounded bg-white/5 px-1.5 py-0.5 text-[9px] text-slate-400"
        :title="`${childCount} 台虚拟机`"
      >
        VM × {{ childCount }}
      </span>
    </div>

    <p class="shrink-0 truncate px-3 pt-1.5 font-mono text-[10px] text-slate-400">
      {{ device.ips[0] ?? '—' }}
    </p>

    <div class="min-h-0 flex-1 rounded-md" />

    <Handle
      id="out"
      type="source"
      :position="Position.Bottom"
      class="!h-2.5 !w-2.5 !border-0 !bg-slate-500 transition hover:!bg-emerald-400"
    />
  </div>

  <!-- Hosted virtual machine: compact card sized by the host layout. -->
  <div
    v-else-if="variant === 'vm'"
    class="flex h-full w-full flex-col justify-center rounded-lg border bg-slate-900/95 px-2.5 py-1.5 shadow-lg shadow-black/20"
    :class="isHighlighted ? 'ring-2 ring-emerald-400/80' : ''"
    :style="{ borderColor: `${color}66` }"
  >
    <Handle
      id="in"
      type="target"
      :position="Position.Top"
      class="!h-2 !w-2 !border-0 !bg-slate-600 transition hover:!bg-emerald-400"
    />

    <div class="flex items-center gap-1.5">
      <span
        class="h-1.5 w-1.5 shrink-0 rounded-full"
        :class="deviceStatusMeta[device.status]?.dot ?? 'bg-slate-500'"
      />
      <span class="truncate text-[11px] font-medium text-slate-100">
        {{ device.name }}
      </span>
      <span v-if="osIcon" class="shrink-0 text-slate-500" :title="osTitle">
        <component :is="osIcon" class="h-3 w-3" />
      </span>
      <span class="ml-auto shrink-0 text-[8px] uppercase tracking-wide text-slate-600">
        VM
      </span>
    </div>
    <p class="mt-0.5 truncate font-mono text-[9px] text-slate-500">
      {{ device.ips[0] ?? '—' }}
      <span v-if="device.model"> · {{ device.model }}</span>
    </p>

    <Handle
      id="out"
      type="source"
      :position="Position.Bottom"
      class="!h-2 !w-2 !border-0 !bg-slate-600 transition hover:!bg-emerald-400"
    />
  </div>

  <!-- Device placed inside a rack container. -->
  <div
    v-else-if="variant === 'racked'"
    class="flex h-full w-full flex-col justify-center rounded-lg border bg-slate-950/95 px-3 py-1.5 shadow-lg shadow-black/15"
    :class="isHighlighted ? 'ring-2 ring-emerald-400/80' : ''"
    :style="{ borderColor: `${color}66` }"
  >
    <Handle
      id="in"
      type="target"
      :position="Position.Top"
      class="!h-2 !w-2 !border-0 !bg-slate-600 transition hover:!bg-emerald-400"
    />

    <div class="flex items-center gap-1.5">
      <span
        class="h-1.5 w-1.5 shrink-0 rounded-full"
        :class="deviceStatusMeta[device.status]?.dot ?? 'bg-slate-500'"
      />
      <span class="truncate text-[11px] font-medium text-slate-100">
        {{ device.name }}
      </span>
      <span v-if="osIcon" class="shrink-0 text-slate-500" :title="osTitle">
        <component :is="osIcon" class="h-3 w-3" />
      </span>
      <span
        class="ml-auto shrink-0 rounded px-1.5 py-0.5 text-[8px]"
        :style="{ color, backgroundColor: `${color}1f` }"
      >
        {{ deviceKindLabel(device.kind) }}
      </span>
    </div>
    <p class="mt-0.5 truncate font-mono text-[9px] text-slate-500">
      {{ device.ips[0] ?? '—' }}
      <span v-if="device.model"> · {{ device.model }}</span>
    </p>

    <Handle
      id="out"
      type="source"
      :position="Position.Bottom"
      class="!h-2 !w-2 !border-0 !bg-slate-600 transition hover:!bg-emerald-400"
    />
  </div>

  <!-- Standalone device. -->
  <div
    v-else
    class="w-52 rounded-xl border bg-slate-950/95 px-3 py-2.5 shadow-xl shadow-black/20"
    :class="isHighlighted ? 'ring-2 ring-emerald-400/80' : ''"
    :style="{ borderColor: `${color}66` }"
  >
    <Handle
      id="in"
      type="target"
      :position="Position.Top"
      class="!h-2.5 !w-2.5 !border-0 !bg-slate-500 transition hover:!bg-emerald-400"
    />

    <div class="flex items-center gap-2">
      <span
        class="h-1.5 w-1.5 rounded-full"
        :class="deviceStatusMeta[device.status]?.dot ?? 'bg-slate-500'"
      />
      <span class="truncate text-xs font-semibold text-slate-100">
        {{ device.name }}
      </span>
      <span v-if="osIcon" class="shrink-0 text-slate-500" :title="osTitle">
        <component :is="osIcon" class="h-3 w-3" />
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
      id="out"
      type="source"
      :position="Position.Bottom"
      class="!h-2.5 !w-2.5 !border-0 !bg-slate-500 transition hover:!bg-emerald-400"
    />
  </div>
</template>
