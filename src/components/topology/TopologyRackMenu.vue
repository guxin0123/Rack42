<script setup lang="ts">
import { Plus } from 'lucide-vue-next'
import { computed, onBeforeUnmount, ref, watch } from 'vue'

import type { Rack } from '@/lib/types'

const props = defineProps<{
  open: boolean
  x: number
  y: number
  rack: Rack | null
  deviceCount: number
}>()

const emit = defineEmits<{
  close: []
  addDevice: []
}>()

const MENU_WIDTH = 236
const ESTIMATED_HEIGHT = 132
const menuRef = ref<HTMLElement | null>(null)

const menuStyle = computed(() => {
  if (typeof window === 'undefined') return {}
  const margin = 8
  const left = Math.min(
    Math.max(margin, props.x),
    Math.max(margin, window.innerWidth - MENU_WIDTH - margin),
  )
  const top = Math.min(
    Math.max(margin, props.y),
    Math.max(margin, window.innerHeight - ESTIMATED_HEIGHT - margin),
  )
  return { left: `${left}px`, top: `${top}px`, width: `${MENU_WIDTH}px` }
})

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
      v-if="open && rack"
      ref="menuRef"
      class="fixed z-40 overflow-y-auto rounded-xl border border-white/10 bg-slate-900 p-1.5 shadow-2xl shadow-black/40"
      :style="menuStyle"
      role="menu"
      @contextmenu.prevent
    >
      <div class="px-2.5 py-2">
        <p class="truncate text-xs font-semibold text-slate-100">{{ rack.name }}</p>
        <p class="mt-0.5 truncate text-[10px] text-slate-500">
          {{ rack.location ?? '未填写位置' }} · {{ rack.totalU }}U ·
          {{ deviceCount }} 台设备
        </p>
      </div>

      <div class="my-1 border-t border-white/5" />

      <button
        type="button"
        role="menuitem"
        class="flex w-full items-center gap-2 rounded-lg px-2.5 py-1.5 text-left text-[11px] text-slate-300 transition hover:bg-white/5 hover:text-slate-100"
        @click="emit('addDevice')"
      >
        <Plus class="h-3.5 w-3.5 shrink-0 text-slate-500" />
        在此机柜添加设备
      </button>
    </div>
  </Teleport>
</template>
