<script setup lang="ts">
import { computed, reactive, ref, watch } from 'vue'

import Modal from '@/components/ui/Modal.vue'
import { api } from '@/lib/ipc'
import type { Credential, CredentialInput } from '@/lib/types'
import { useAppStore } from '@/stores/app'

const props = defineProps<{
  open: boolean
  deviceId?: number | null
  primaryIp: string
  credential?: Credential | null
  /** `draft` collects the form value without touching the backend. */
  mode?: 'remote' | 'draft'
}>()

const emit = defineEmits<{
  close: []
  saved: []
  save: [input: CredentialInput]
}>()

const store = useAppStore()
const busy = ref(false)
const formError = ref('')

const form = reactive({
  typeId: '' as string,
  label: '',
  followPrimary: true,
  address: '',
  port: '' as string,
  username: '',
  domain: '',
  password: '',
  clearPassword: false,
  note: '',
  isDefault: false,
})

const selectedType = computed(() =>
  store.credentialTypes.find((type) => String(type.id) === form.typeId),
)

const resolvedAddress = computed(() => {
  if (form.followPrimary) return props.primaryIp || '设备主 IP'
  return form.address.trim() || '未填写'
})

watch(
  () => props.open,
  (open) => {
    if (!open) return
    const credential = props.credential
    formError.value = ''
    form.typeId = String(
      credential?.typeId ??
        store.credentialTypes.find((type) => type.slug === 'rdp')?.id ??
        store.credentialTypes[0]?.id ??
        '',
    )
    form.label = credential?.label ?? ''
    form.followPrimary = !credential?.address
    form.address = credential?.address ?? ''
    form.port = credential?.port ? String(credential.port) : ''
    form.username = credential?.username ?? ''
    form.domain = credential?.domain ?? ''
    form.password = ''
    form.clearPassword = false
    form.note = credential?.note ?? ''
    form.isDefault = credential?.isDefault ?? false
  },
  { immediate: true },
)

async function submit(): Promise<void> {
  if (!form.typeId) {
    formError.value = '请选择凭据类型'
    return
  }

  const port = form.port ? Number(form.port) : null
  if (port !== null && (!Number.isInteger(port) || port < 1 || port > 65535)) {
    formError.value = '端口需要在 1 到 65535 之间'
    return
  }

  let password: string | null = null
  if (props.credential) {
    if (form.clearPassword) password = ''
    else if (form.password) password = form.password
  } else if (form.password) {
    password = form.password
  }

  const input: CredentialInput = {
    deviceId: props.deviceId ?? 0,
    typeId: Number(form.typeId),
    label: form.label.trim() || null,
    address: form.followPrimary ? null : form.address.trim() || null,
    port,
    username: form.username.trim() || null,
    domain: form.domain.trim() || null,
    password,
    note: form.note.trim() || null,
    isDefault: form.isDefault,
  }

  if (props.mode === 'draft') {
    emit('save', input)
    emit('close')
    return
  }

  busy.value = true
  formError.value = ''
  try {
    if (props.credential) {
      await api.updateCredential(props.credential.id, input)
      store.notify('凭据已更新')
    } else {
      await api.createCredential(input)
      store.notify('凭据已添加')
    }
    // Do not keep the dialog busy while the whole store refreshes.
    emit('saved')
    emit('close')
    void store
      .refresh()
      .catch((caught) =>
        store.notify(caught instanceof Error ? caught.message : '刷新数据失败'),
      )
  } catch (caught) {
    formError.value = caught instanceof Error ? caught.message : '保存失败'
  } finally {
    busy.value = false
  }
}
</script>

<template>
  <Modal
    :open="open"
    :title="credential ? '编辑凭据' : '新增凭据'"
    width="max-w-2xl"
    @close="$emit('close')"
  >
    <form class="space-y-4" @submit.prevent="submit">
      <div class="grid grid-cols-2 gap-4">
        <div>
          <label class="label" for="device-credential-type">凭据类型</label>
          <select id="device-credential-type" v-model="form.typeId" class="input">
            <option v-for="type in store.credentialTypes" :key="type.id" :value="String(type.id)">
              {{ type.name }}
            </option>
          </select>
        </div>
        <div>
          <label class="label" for="device-credential-label">标签</label>
          <input
            id="device-credential-label"
            v-model="form.label"
            class="input"
            placeholder="如 管理口 / root"
            maxlength="40"
          />
        </div>
      </div>

      <div>
        <label class="label">地址</label>
        <label class="mb-2 flex items-center gap-2 text-[11px] text-slate-400">
          <input v-model="form.followPrimary" type="checkbox" class="accent-emerald-500" />
          跟随设备主 IP
          <span class="font-mono text-[10px] text-slate-600">{{ resolvedAddress }}</span>
        </label>
        <input
          v-if="!form.followPrimary"
          v-model="form.address"
          class="input"
          placeholder="如 10.0.0.21 或 https://10.0.0.21"
          autocomplete="off"
        />
      </div>

      <div class="grid grid-cols-3 gap-4">
        <div class="col-span-1">
          <label class="label" for="device-credential-port">端口</label>
          <input
            id="device-credential-port"
            v-model="form.port"
            class="input"
            type="number"
            min="1"
            max="65535"
            :placeholder="selectedType?.defaultPort ? `默认 ${selectedType.defaultPort}` : '默认'"
          />
        </div>
        <div class="col-span-2">
          <label class="label" for="device-credential-username">用户名</label>
          <input
            id="device-credential-username"
            v-model="form.username"
            class="input"
            autocomplete="off"
          />
        </div>
      </div>

      <div class="grid grid-cols-2 gap-4">
        <div>
          <label class="label" for="device-credential-domain">域（RDP 可选）</label>
          <input
            id="device-credential-domain"
            v-model="form.domain"
            class="input"
            placeholder="如 CORP"
            autocomplete="off"
          />
        </div>
        <div>
          <label class="label" for="device-credential-password">密码</label>
          <input
            id="device-credential-password"
            v-model="form.password"
            class="input"
            type="password"
            autocomplete="new-password"
            :disabled="form.clearPassword"
            :placeholder="
              credential
                ? '留空表示不修改'
                : '留空表示连接时由客户端询问'
            "
          />
        </div>
      </div>

      <label v-if="credential" class="flex items-center gap-2 text-[11px] text-slate-500">
        <input v-model="form.clearPassword" type="checkbox" class="accent-emerald-500" />
        清除已保存的密码
      </label>

      <label class="flex items-center gap-2 text-[11px] text-slate-400">
        <input v-model="form.isDefault" type="checkbox" class="accent-emerald-500" />
        设为此设备该类型的默认凭据
      </label>

      <div>
        <label class="label" for="device-credential-note">备注</label>
        <textarea
          id="device-credential-note"
          v-model="form.note"
          class="input resize-none"
          rows="2"
        />
      </div>

      <p
        v-if="formError"
        class="rounded-lg border border-rose-500/20 bg-rose-500/10 px-3 py-2 text-xs text-rose-300"
      >
        {{ formError }}
      </p>
    </form>

    <template #footer>
      <button class="btn btn-ghost" @click="$emit('close')">取消</button>
      <button class="btn btn-primary" :disabled="busy" @click="submit">
        {{ busy ? '保存中…' : '保存' }}
      </button>
    </template>
  </Modal>
</template>
