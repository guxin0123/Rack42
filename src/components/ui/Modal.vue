<script setup lang="ts">
import { X } from 'lucide-vue-next'

withDefaults(
  defineProps<{
    open: boolean
    title: string
    width?: string
  }>(),
  { width: 'max-w-lg' },
)

defineEmits<{ close: [] }>()
</script>

<template>
  <Teleport to="body">
    <Transition
      enter-active-class="transition duration-150 ease-out"
      enter-from-class="opacity-0"
      enter-to-class="opacity-100"
      leave-active-class="transition duration-100 ease-in"
      leave-from-class="opacity-100"
      leave-to-class="opacity-0"
    >
      <div
        v-if="open"
        class="modal-backdrop fixed inset-0 z-50 flex items-center justify-center bg-slate-950/70 p-4 backdrop-blur-sm"
        @click.self="$emit('close')"
      >
        <div
          class="flex max-h-[calc(100vh-2rem)] w-full flex-col overflow-hidden rounded-2xl border border-white/10 bg-slate-900 shadow-2xl"
          :class="width"
        >
          <header
            class="flex shrink-0 items-center justify-between border-b border-white/10 px-5 py-4"
          >
            <h2 class="text-sm font-semibold tracking-wide text-slate-100">
              {{ title }}
            </h2>
            <button
              class="rounded-md p-1 text-slate-400 transition hover:bg-white/5 hover:text-slate-200"
              type="button"
              aria-label="关闭"
              @click="$emit('close')"
            >
              <X class="h-4 w-4" />
            </button>
          </header>

          <div class="min-h-0 flex-1 overflow-y-auto px-5 py-5">
            <slot />
          </div>

          <footer
            v-if="$slots.footer"
            class="flex shrink-0 items-center justify-end gap-2 border-t border-white/10 bg-slate-950/40 px-5 py-3"
          >
            <slot name="footer" />
          </footer>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>
