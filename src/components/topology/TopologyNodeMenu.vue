<script setup lang="ts">
import {
  FolderUp,
  Globe,
  KeyRound,
  Link2,
  Monitor,
  Pencil,
  Terminal,
  Trash2,
} from 'lucide-vue-next'
import { computed, onBeforeUnmount, ref, watch } from 'vue'

import { actionLabels } from '@/composables/useDeviceConnect'
import {
  deviceKindLabel,
  deviceStatusLabel,
} from '@/lib/format'
import type {
  ConnectionTypeSummary,
  CredentialAction,
  Device,
} from '@/lib/types'

const props = defineProps<{
  open: boolean
  x: number
  y: number
  device: Device | null
  types: ConnectionTypeSummary[]
  busy: boolean
}>()

const emit = defineEmits<{
  close: []
  connect: [type: ConnectionTypeSummary, action: CredentialAction]
  edit: []
  credentials: []
  addLink: []
  remove: []
}>()

const MENU_WIDTH = 252
const menuRef = ref<HTMLElement | null>(null)

const actionCount = computed(() =>
  props.types.reduce((total, type) => total + type.actions.length, 0),
)

const estimatedHeight = computed(
  () => 62 + (actionCount.value ? actionCount.value * 34 + 26 : 40) + 4 * 34 + 18,
)

const menuStyle = computed(() => {
  if (typeof window === 'undefined') return {}
  const margin = 8
  const left = Math.min(
    Math.max(margin, props.x),
    Math.max(margin, window.innerWidth - MENU_WIDTH - margin),
  )
  const top = Math.min(
    Math.max(margin, props.y),
    Math.max(margin, window.innerHeight - estimatedHeight.value - margin),
  )
  return {
    left: `${left}px`,
    top: `${top}px`,
    width: `${MENU_WIDTH}px`,
    maxHeight: `calc(100vh - ${top + margin}px)`,
  }
})

function actionIcon(action: CredentialAction) {
  if (action === 'rdp') return Monitor
  if (action === 'web') return Globe
  if (action === 'ssh') return Terminal
  return FolderUp
}

function close(): void {
  if (props.open) emit('close')
}

function onPointerDown(event: PointerEvent): void {
  if (menuRef.value?.contains(event.target as Node)) return
  close()
}

function onKeyDown(event: KeyboardEvent): void {
  if (event.key === 'Escape') close()
}

function onWheel(event: WheelEvent): void {
  if (menuRef.value?.contains(event.target as Node)) return
  close()
}

function addListeners(): void {
  window.addEventListener('pointerdown', onPointerDown, true)
  window.addEventListener('keydown', onKeyDown)
  window.addEventListener('wheel', onWheel, { passive: true })
  window.addEventListener('resize', close)
  window.addEventListener('blur', close)
}

function removeListeners(): void {
  window.removeEventListener('pointerdown', onPointerDown, true)
  window.removeEventListener('keydown', onKeyDown)
  window.removeEventListener('wheel', onWheel)
  window.removeEventListener('resize', close)
  window.removeEventListener('blur', close)
}

watch(
  () => props.open,
  (open) => {
    if (open) addListeners()
    else removeListeners()
  },
  { immediate: true },
)

onBeforeUnmount(removeListeners)
</script>

<template>
  <Teleport to="body">
    <div
      v-if="open && device"
      ref="menuRef"
      class="fixed z-40 overflow-y-auto rounded-xl border border-white/10 bg-slate-900 p-1.5 shadow-2xl shadow-black/40"
      :style="menuStyle"
      role="menu"
      @contextmenu.prevent
    >
      <div class="px-2.5 py-2">
        <p class="truncate text-xs font-semibold text-slate-100">{{ device.name }}</p>
        <p class="mt-0.5 truncate text-[10px] text-slate-500">
          {{ deviceKindLabel(device.kind) }} · {{ deviceStatusLabel(device.status) }}
          <template v-if="device.ips[0]"> · {{ device.ips[0] }}</template>
        </p>
      </div>

      <p class="px-2.5 pb-1 pt-1.5 text-[10px] tracking-wide text-slate-600">连接</p>

      <template v-if="types.length">
        <template v-for="type in types" :key="type.typeId">
          <button
            v-for="action in type.actions"
            :key="`${type.typeId}-${action}`"
            type="button"
            role="menuitem"
            class="flex w-full items-center gap-2 rounded-lg px-2.5 py-1.5 text-left text-[11px] text-slate-300 transition hover:bg-emerald-500/10 hover:text-emerald-200 disabled:cursor-not-allowed disabled:opacity-40"
            :disabled="busy"
            @click="emit('connect', type, action)"
          >
            <component :is="actionIcon(action)" class="h-3.5 w-3.5 shrink-0" />
            <span class="truncate">{{ type.name }} · {{ actionLabels[action] ?? action }}</span>
            <span
              v-if="type.credentialCount > 1"
              class="ml-auto shrink-0 rounded bg-white/5 px-1.5 py-0.5 text-[9px] text-slate-500"
            >
              {{ type.credentialCount }} 条
            </span>
          </button>
        </template>
      </template>
      <p v-else class="px-2.5 py-2 text-[11px] text-slate-500">
        该设备还没有可连接的凭据
      </p>

      <div class="my-1 border-t border-white/5" />

      <button
        type="button"
        role="menuitem"
        class="flex w-full items-center gap-2 rounded-lg px-2.5 py-1.5 text-left text-[11px] text-slate-300 transition hover:bg-white/5 hover:text-slate-100"
        @click="emit('addLink')"
      >
        <Link2 class="h-3.5 w-3.5 shrink-0 text-slate-500" />
        以该设备为源添加连接
      </button>
      <button
        type="button"
        role="menuitem"
        class="flex w-full items-center gap-2 rounded-lg px-2.5 py-1.5 text-left text-[11px] text-slate-300 transition hover:bg-white/5 hover:text-slate-100"
        @click="emit('credentials')"
      >
        <KeyRound class="h-3.5 w-3.5 shrink-0 text-slate-500" />
        管理凭据
      </button>
      <button
        type="button"
        role="menuitem"
        class="flex w-full items-center gap-2 rounded-lg px-2.5 py-1.5 text-left text-[11px] text-slate-300 transition hover:bg-white/5 hover:text-slate-100"
        @click="emit('edit')"
      >
        <Pencil class="h-3.5 w-3.5 shrink-0 text-slate-500" />
        编辑设备
      </button>
      <button
        type="button"
        role="menuitem"
        class="flex w-full items-center gap-2 rounded-lg px-2.5 py-1.5 text-left text-[11px] text-rose-300 transition hover:bg-rose-500/10 hover:text-rose-200"
        @click="emit('remove')"
      >
        <Trash2 class="h-3.5 w-3.5 shrink-0" />
        删除设备
      </button>
    </div>
  </Teleport>
</template>
