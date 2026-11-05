<script setup lang="ts">
import Modal from '@/components/ui/Modal.vue'
import { actionLabels, type CredentialChoice } from '@/composables/useDeviceConnect'
import type { ConnectionCredentialSummary } from '@/lib/types'

defineProps<{
  open: boolean
  chooser: CredentialChoice | null
  deviceName?: string
}>()

const emit = defineEmits<{
  close: []
  select: [credential: ConnectionCredentialSummary]
}>()
</script>

<template>
  <Modal :open="open" title="选择凭据" width="max-w-md" @close="emit('close')">
    <div v-if="chooser" class="space-y-2">
      <p class="mb-3 text-[11px] text-slate-500">
        <template v-if="deviceName">{{ deviceName }} · </template>
        {{ chooser.type.name }} · {{ actionLabels[chooser.action] ?? chooser.action }}
      </p>
      <button
        v-for="credential in chooser.type.credentials"
        :key="credential.id"
        type="button"
        class="flex w-full items-center justify-between gap-3 rounded-xl border border-white/5 bg-white/[0.02] px-3 py-2.5 text-left transition hover:border-emerald-400/30 hover:bg-emerald-400/5"
        @click="emit('select', credential)"
      >
        <span class="min-w-0">
          <span class="block truncate text-xs font-medium text-slate-200">
            {{ credential.label ?? '未命名' }}
          </span>
          <span class="mt-0.5 block truncate font-mono text-[10px] text-slate-500">
            {{ credential.username || '无用户名' }}
          </span>
        </span>
        <span
          v-if="credential.isDefault"
          class="shrink-0 rounded bg-emerald-500/10 px-1.5 py-0.5 text-[9px] text-emerald-300"
        >
          默认
        </span>
      </button>
    </div>
    <template #footer>
      <button class="btn btn-ghost" type="button" @click="emit('close')">取消</button>
    </template>
  </Modal>
</template>
