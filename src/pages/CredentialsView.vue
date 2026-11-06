<script setup lang="ts">
import {
  Copy,
  Eye,
  EyeOff,
  KeyRound,
  Pencil,
  Plus,
  Trash2,
} from 'lucide-vue-next'
import { computed, reactive, ref, watch } from 'vue'

import Modal from '@/components/ui/Modal.vue'
import { api } from '@/lib/ipc'
import type { Credential, CredentialInput } from '@/lib/types'
import { useAppStore } from '@/stores/app'

const store = useAppStore()

const selectedDeviceId = ref<number | null>(store.devices[0]?.id ?? null)
const credentials = ref<Credential[]>([])
const loading = ref(false)
const revealed = reactive<Record<number, string>>({})

const modalOpen = ref(false)
const editing = ref<Credential | null>(null)
const busiest = ref(false)
const formError = ref('')

const form = reactive({
  typeId: '' as string,
  label: '',
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

const devicePrimaryIp = computed(() => {
  const device = selectedDeviceId.value
    ? store.deviceById.get(selectedDeviceId.value)
    : undefined
  return device?.ips[0] ?? ''
})

watch(
  () => store.devices.map((device) => device.id).join(','),
  () => {
    if (!store.devices.some((device) => device.id === selectedDeviceId.value)) {
      selectedDeviceId.value = store.devices[0]?.id ?? null
    }
  },
)

watch(
  selectedDeviceId,
  (deviceId) => {
    for (const key of Object.keys(revealed)) delete revealed[Number(key)]
    if (deviceId) {
      void loadCredentials(deviceId)
    } else {
      credentials.value = []
    }
  },
  { immediate: true },
)

async function loadCredentials(deviceId: number): Promise<void> {
  loading.value = true
  try {
    credentials.value = await api.listCredentials(deviceId)
  } catch (caught) {
    store.notify(caught instanceof Error ? caught.message : '加载凭据失败')
  } finally {
    loading.value = false
  }
}

function typeFor(typeId: number) {
  return store.credentialTypes.find((type) => type.id === typeId)
}

function typeName(typeId: number): string {
  return typeFor(typeId)?.name ?? '其他'
}

function typePort(typeId: number): number | null {
  return typeFor(typeId)?.defaultPort ?? null
}

function effectiveAddress(credential: Credential): string {
  if (credential.address) return credential.address
  const device = store.deviceById.get(credential.deviceId)
  return device?.ips[0] ?? '—'
}

function openCreate(): void {
  editing.value = null
  form.typeId = String(store.credentialTypes[0]?.id ?? '')
  form.label = ''
  form.address = ''
  form.port = ''
  form.username = ''
  form.domain = ''
  form.password = ''
  form.clearPassword = false
  form.note = ''
  form.isDefault = false
  formError.value = ''
  modalOpen.value = true
}

function openEdit(credential: Credential): void {
  editing.value = credential
  form.typeId = String(credential.typeId)
  form.label = credential.label ?? ''
  form.address = credential.address ?? ''
  form.port = credential.port ? String(credential.port) : ''
  form.username = credential.username ?? ''
  form.domain = credential.domain ?? ''
  form.password = ''
  form.clearPassword = false
  form.note = credential.note ?? ''
  form.isDefault = credential.isDefault
  formError.value = ''
  modalOpen.value = true
}

async function submit(): Promise<void> {
  if (!selectedDeviceId.value) return
  if (!form.typeId) {
    formError.value = '请选择凭据类型'
    return
  }

  busiest.value = true
  formError.value = ''
  try {
    let password: string | null = null
    if (editing.value) {
      if (form.clearPassword) password = ''
      else if (form.password) password = form.password
    } else if (form.password) {
      password = form.password
    }

    const input: CredentialInput = {
      deviceId: selectedDeviceId.value,
      typeId: Number(form.typeId),
      label: form.label.trim() || null,
      address: form.address.trim() || null,
      port: form.port ? Number(form.port) : null,
      username: form.username.trim() || null,
      domain: form.domain.trim() || null,
      password,
      note: form.note.trim() || null,
      isDefault: form.isDefault,
    }

    if (editing.value) {
      await api.updateCredential(editing.value.id, input)
      store.notify('凭据已更新')
    } else {
      await api.createCredential(input)
      store.notify('凭据已保存')
    }
    // Close the dialog before the global refresh so a slow refresh never
    // makes the save action look stuck.
    modalOpen.value = false
    await loadCredentials(selectedDeviceId.value)
    void store
      .refresh()
      .catch((caught) =>
        store.notify(caught instanceof Error ? caught.message : '刷新数据失败'),
      )
  } catch (caught) {
    formError.value = caught instanceof Error ? caught.message : '保存失败'
  } finally {
    busiest.value = false
  }
}

async function removeCredential(credential: Credential): Promise<void> {
  if (!window.confirm(`确定删除凭据「${credential.label ?? credential.username ?? '未命名'}」吗？`)) {
    return
  }
  try {
    await api.deleteCredential(credential.id)
    store.notify('凭据已删除')
    if (selectedDeviceId.value) await loadCredentials(selectedDeviceId.value)
    await store.refresh()
  } catch (caught) {
    store.notify(caught instanceof Error ? caught.message : '删除失败')
  }
}

async function toggleReveal(credential: Credential): Promise<void> {
  if (!credential.hasPassword) return
  if (revealed[credential.id]) {
    delete revealed[credential.id]
    return
  }
  try {
    revealed[credential.id] = await api.revealCredential(credential.id)
  } catch (caught) {
    store.notify(caught instanceof Error ? caught.message : '无法读取密码')
  }
}

async function copyPassword(credential: Credential): Promise<void> {
  if (!credential.hasPassword) {
    store.notify('该凭据没有保存密码')
    return
  }
  try {
    const value = revealed[credential.id] ?? (await api.revealCredential(credential.id))
    await navigator.clipboard.writeText(value)
    store.notify('密码已复制到剪贴板')
  } catch (caught) {
    store.notify(caught instanceof Error ? caught.message : '复制失败')
  }
}
</script>

<template>
  <div class="space-y-5">
    <div class="flex flex-wrap items-end justify-between gap-3">
      <div>
        <h2 class="text-base font-semibold tracking-tight text-slate-100">凭据</h2>
        <p class="mt-1 text-xs text-slate-500">
          按类型管理地址、端口与账号；密码经 Argon2id 派生密钥加密，界面默认掩码
        </p>
      </div>
      <div class="flex items-center gap-2">
        <select v-model="selectedDeviceId" class="input w-56">
          <option :value="null" disabled>选择设备</option>
          <option v-for="device in store.devices" :key="device.id" :value="device.id">
            {{ device.name }}
          </option>
        </select>
        <button class="btn btn-primary" :disabled="!selectedDeviceId" @click="openCreate">
          <Plus class="h-3.5 w-3.5" />
          新增凭据
        </button>
      </div>
    </div>

    <div
      v-if="!store.devices.length"
      class="card flex flex-col items-center gap-3 px-6 py-16 text-center"
    >
      <KeyRound class="h-7 w-7 text-slate-600" />
      <p class="text-sm font-medium text-slate-300">还没有设备</p>
      <p class="max-w-sm text-xs text-slate-500">
        凭据需要绑定到具体设备，请先在「设备」页面添加设备。
      </p>
    </div>

    <div v-else class="card overflow-hidden">
      <div class="flex items-center justify-between border-b border-white/5 px-5 py-3.5">
        <p class="text-[11px] text-slate-500">
          {{ loading ? '加载中…' : `${credentials.length} 条凭据` }}
        </p>
        <p class="text-[10px] text-slate-600">地址留空时自动使用设备主 IP</p>
      </div>

      <div class="overflow-x-auto">
        <table v-if="credentials.length" class="w-full min-w-[56rem] text-left text-xs">
          <thead class="text-[10px] uppercase tracking-wider text-slate-500">
            <tr class="border-b border-white/5">
              <th class="px-5 py-3 font-medium">类型</th>
              <th class="px-5 py-3 font-medium">标签</th>
              <th class="px-5 py-3 font-medium">地址</th>
              <th class="px-5 py-3 font-medium">用户名</th>
              <th class="px-5 py-3 font-medium">端口</th>
              <th class="px-5 py-3 font-medium">密码</th>
              <th class="px-5 py-3 font-medium">备注</th>
              <th class="px-5 py-3 text-right font-medium">操作</th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="credential in credentials"
              :key="credential.id"
              class="border-b border-white/[0.03] transition last:border-0 hover:bg-white/[0.02]"
            >
              <td class="px-5 py-3">
                <span class="badge bg-white/5 text-slate-300 ring-white/10">
                  {{ typeName(credential.typeId) }}
                </span>
                <span
                  v-if="credential.isDefault"
                  class="ml-1.5 rounded bg-emerald-500/10 px-1.5 py-0.5 text-[9px] text-emerald-300"
                >
                  默认
                </span>
              </td>
              <td class="px-5 py-3 font-medium text-slate-200">
                {{ credential.label ?? '未命名' }}
              </td>
              <td class="px-5 py-3 font-mono text-[11px] text-slate-400">
                {{ effectiveAddress(credential) }}
              </td>
              <td class="px-5 py-3 font-mono text-[11px] text-slate-400">
                {{ credential.username ?? '—' }}
              </td>
              <td class="px-5 py-3 font-mono text-[11px] text-slate-400">
                {{ credential.port ?? typePort(credential.typeId) ?? '—' }}
              </td>
              <td class="px-5 py-3">
                <span
                  class="font-mono text-[11px]"
                  :class="revealed[credential.id] ? 'text-emerald-300' : 'text-slate-500'"
                >
                  {{
                    !credential.hasPassword
                      ? '未保存'
                      : (revealed[credential.id] ?? '••••••••')
                  }}
                </span>
              </td>
              <td class="max-w-48 truncate px-5 py-3 text-slate-500">
                {{ credential.note ?? '—' }}
              </td>
              <td class="px-5 py-3">
                <div class="flex justify-end gap-1">
                  <button
                    class="rounded-md p-1.5 text-slate-400 transition hover:bg-white/10 hover:text-slate-200 disabled:opacity-30"
                    :title="revealed[credential.id] ? '隐藏' : '显示'"
                    :disabled="!credential.hasPassword"
                    @click="toggleReveal(credential)"
                  >
                    <EyeOff v-if="revealed[credential.id]" class="h-3.5 w-3.5" />
                    <Eye v-else class="h-3.5 w-3.5" />
                  </button>
                  <button
                    class="rounded-md p-1.5 text-slate-400 transition hover:bg-white/10 hover:text-slate-200 disabled:opacity-30"
                    title="复制密码"
                    :disabled="!credential.hasPassword"
                    @click="copyPassword(credential)"
                  >
                    <Copy class="h-3.5 w-3.5" />
                  </button>
                  <button
                    class="rounded-md p-1.5 text-slate-400 transition hover:bg-white/10 hover:text-slate-200"
                    title="编辑"
                    @click="openEdit(credential)"
                  >
                    <Pencil class="h-3.5 w-3.5" />
                  </button>
                  <button
                    class="rounded-md p-1.5 text-slate-500 transition hover:bg-rose-500/10 hover:text-rose-300"
                    title="删除"
                    @click="removeCredential(credential)"
                  >
                    <Trash2 class="h-3.5 w-3.5" />
                  </button>
                </div>
              </td>
            </tr>
          </tbody>
        </table>
      </div>

      <div v-if="!credentials.length" class="flex flex-col items-center gap-2 px-6 py-12 text-center">
        <p class="text-xs text-slate-400">该设备还没有凭据</p>
        <button class="btn btn-ghost mt-1" @click="openCreate">
          <Plus class="h-3.5 w-3.5" />
          添加第一条
        </button>
      </div>
    </div>

    <Modal
      :open="modalOpen"
      :title="editing ? '编辑凭据' : '新增凭据'"
      width="max-w-2xl"
      @close="modalOpen = false"
    >
      <form class="space-y-4" @submit.prevent="submit">
        <div class="grid grid-cols-2 gap-4">
          <div>
            <label class="label" for="credential-type">凭据类型</label>
            <select id="credential-type" v-model="form.typeId" class="input">
              <option v-for="type in store.credentialTypes" :key="type.id" :value="String(type.id)">
                {{ type.name }}
              </option>
            </select>
          </div>
          <div>
            <label class="label" for="credential-label">标签</label>
            <input
              id="credential-label"
              v-model="form.label"
              class="input"
              placeholder="如 管理口 / root / Web 管理"
              maxlength="40"
            />
          </div>
        </div>

        <div class="grid grid-cols-3 gap-4">
          <div class="col-span-2">
            <label class="label" for="credential-address">地址</label>
            <input
              id="credential-address"
              v-model="form.address"
              class="input"
              :placeholder="devicePrimaryIp ? `留空使用主 IP：${devicePrimaryIp}` : '留空使用设备主 IP'"
              autocomplete="off"
            />
          </div>
          <div>
            <label class="label" for="credential-port">端口</label>
            <input
              id="credential-port"
              v-model="form.port"
              class="input"
              type="number"
              min="1"
              max="65535"
              :placeholder="
                selectedType?.defaultPort
                  ? `默认 ${selectedType.defaultPort}`
                  : '默认端口'
              "
            />
          </div>
        </div>

        <div class="grid grid-cols-2 gap-4">
          <div>
            <label class="label" for="credential-username">用户名</label>
            <input
              id="credential-username"
              v-model="form.username"
              class="input"
              autocomplete="off"
            />
          </div>
          <div>
            <label class="label" for="credential-domain">域（RDP 可选）</label>
            <input
              id="credential-domain"
              v-model="form.domain"
              class="input"
              placeholder="如 CORP"
              autocomplete="off"
            />
          </div>
        </div>

        <div>
          <label class="label" for="credential-password">密码</label>
          <input
            id="credential-password"
            v-model="form.password"
            class="input"
            type="password"
            autocomplete="new-password"
            :disabled="form.clearPassword"
            :placeholder="
              editing
                ? '留空表示不修改；勾选下方可清除'
                : '留空表示不保存密码，连接时由客户端询问'
            "
          />
          <label
            v-if="editing"
            class="mt-2 flex items-center gap-2 text-[11px] text-slate-500"
          >
            <input v-model="form.clearPassword" type="checkbox" class="accent-emerald-500" />
            清除已保存的密码
          </label>
        </div>

        <label class="flex items-center gap-2 text-[11px] text-slate-400">
          <input v-model="form.isDefault" type="checkbox" class="accent-emerald-500" />
          设为此设备该类型的默认凭据
        </label>

        <div>
          <label class="label" for="credential-note">备注</label>
          <textarea
            id="credential-note"
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
        <button class="btn btn-ghost" @click="modalOpen = false">取消</button>
        <button class="btn btn-primary" :disabled="busiest" @click="submit">
          {{ busiest ? '保存中…' : '保存' }}
        </button>
      </template>
    </Modal>
  </div>
</template>
