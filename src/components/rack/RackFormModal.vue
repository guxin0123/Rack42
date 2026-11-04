<script setup lang="ts">
import { reactive, ref, watch } from 'vue'

import Modal from '@/components/ui/Modal.vue'
import type { Rack } from '@/lib/types'
import { useAppStore } from '@/stores/app'

const props = defineProps<{
  open: boolean
  rack?: Rack | null
}>()

const emit = defineEmits<{ close: []; saved: [] }>()

const store = useAppStore()
const form = reactive({
  name: '',
  location: '',
  totalU: 42,
  note: '',
})
const error = ref('')
const busy = ref(false)

watch(
  () => props.open,
  (open) => {
    if (!open) return
    error.value = ''
    form.name = props.rack?.name ?? ''
    form.location = props.rack?.location ?? ''
    form.totalU = props.rack?.totalU ?? 42
    form.note = props.rack?.note ?? ''
  },
  { immediate: true },
)

async function submit(): Promise<void> {
  if (!form.name.trim()) {
    error.value = '请输入机柜名称'
    return
  }

  busy.value = true
  error.value = ''
  try {
    const payload = {
      name: form.name.trim(),
      location: form.location.trim() || null,
      totalU: Number(form.totalU) || 42,
      note: form.note.trim() || null,
    }
    if (props.rack) {
      await store.updateRack(props.rack.id, payload)
      store.notify('机柜已更新')
    } else {
      await store.createRack(payload)
      store.notify('机柜已创建')
    }
    emit('saved')
    emit('close')
  } catch (caught) {
    error.value = caught instanceof Error ? caught.message : '保存失败'
  } finally {
    busy.value = false
  }
}
</script>

<template>
  <Modal
    :open="open"
    :title="rack ? '编辑机柜' : '新建机柜'"
    width="max-w-md"
    @close="$emit('close')"
  >
    <form class="space-y-4" @submit.prevent="submit">
      <div>
        <label class="label" for="rack-name">机柜名称</label>
        <input
          id="rack-name"
          v-model="form.name"
          class="input"
          placeholder="如 A01"
          maxlength="40"
        />
      </div>

      <div class="grid grid-cols-2 gap-3">
        <div>
          <label class="label" for="rack-location">位置</label>
          <input
            id="rack-location"
            v-model="form.location"
            class="input"
            placeholder="如 3F 机房"
            maxlength="60"
          />
        </div>
        <div>
          <label class="label" for="rack-total-u">总 U 数</label>
          <input
            id="rack-total-u"
            v-model.number="form.totalU"
            class="input"
            type="number"
            min="1"
            max="48"
          />
        </div>
      </div>

      <div>
        <label class="label" for="rack-note">备注</label>
        <textarea
          id="rack-note"
          v-model="form.note"
          class="input resize-none"
          rows="3"
          placeholder="供电、承重、网络接入等说明"
        />
      </div>

      <p
        v-if="error"
        class="rounded-lg border border-rose-500/20 bg-rose-500/10 px-3 py-2 text-xs text-rose-300"
      >
        {{ error }}
      </p>
    </form>

    <template #footer>
      <button class="btn btn-ghost" type="button" @click="$emit('close')">取消</button>
      <button class="btn btn-primary" type="button" :disabled="busy" @click="submit">
        {{ busy ? '保存中…' : rack ? '保存修改' : '创建机柜' }}
      </button>
    </template>
  </Modal>
</template>
