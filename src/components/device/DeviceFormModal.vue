<script setup lang="ts">
import { KeyRound, Pencil, Plus, Trash2 } from 'lucide-vue-next'
import { computed, reactive, ref, watch } from 'vue'

import CredentialFormModal from '@/components/credential/CredentialFormModal.vue'
import Modal from '@/components/ui/Modal.vue'
import { deviceKindMeta } from '@/lib/format'
import { api } from '@/lib/ipc'
import {
  osFamilyGroups,
  osFamilyOf,
  osOptionsFor,
} from '@/lib/os'
import type {
  Credential,
  Device,
  DeviceKind,
  DeviceStatus,
  OsFamily,
} from '@/lib/types'
import { useAppStore } from '@/stores/app'

const props = defineProps<{
  open: boolean
  device?: Device | null
  defaultRackId?: number | null
  defaultPosition?: number | null
  initialTab?: 'basic' | 'credentials'
}>()

const emit = defineEmits<{ close: []; saved: [] }>()

const store = useAppStore()
const busy = ref(false)
const error = ref('')
const activeTab = ref<'basic' | 'credentials'>('basic')

const credentials = ref<Credential[]>([])
const credentialsLoading = ref(false)
const credentialsDeviceId = ref<number | null>(null)
const credentialModalOpen = ref(false)
const editingCredential = ref<Credential | null>(null)

const kinds = Object.entries(deviceKindMeta) as Array<
  [DeviceKind, (typeof deviceKindMeta)[DeviceKind]]
>

const statuses: Array<{ value: DeviceStatus; label: string }> = [
  { value: 'active', label: '运行中' },
  { value: 'maintenance', label: '维护中' },
  { value: 'offline', label: '已离线' },
  { value: 'retired', label: '已退役' },
]

const form = reactive({
  name: '',
  kind: 'physical_server' as DeviceKind,
  status: 'active' as DeviceStatus,
  rackId: '',
  rackPosition: '',
  rackHeight: '1',
  ips: [''] as string[],
  vendor: '',
  model: '',
  serial: '',
  osFamily: 'other' as OsFamily,
  os: '',
  hostId: '',
  note: '',
})

const hostCandidates = computed(() =>
  store.devices.filter(
    (device) => device.kind === 'physical_server' && device.id !== props.device?.id,
  ),
)

const currentOsOptions = computed(() => {
  const options = osOptionsFor(form.osFamily)
  if (form.os && !options.includes(form.os)) {
    return [form.os, ...options]
  }
  return options
})

watch(
  () => props.open,
  (open) => {
    if (!open) {
      credentialModalOpen.value = false
      editingCredential.value = null
      return
    }
    error.value = ''
    const device = props.device
    form.name = device?.name ?? ''
    form.kind = device?.kind ?? 'physical_server'
    form.status = device?.status ?? 'active'
    form.rackId = device?.rackId
      ? String(device.rackId)
      : props.defaultRackId
        ? String(props.defaultRackId)
        : ''
    form.rackPosition = device?.rackPosition
      ? String(device.rackPosition)
      : props.defaultPosition
        ? String(props.defaultPosition)
        : ''
    form.rackHeight = String(device?.rackHeight ?? 1)
    form.ips = device?.ips?.length ? [...device.ips] : ['']
    form.vendor = device?.vendor ?? ''
    form.model = device?.model ?? ''
    form.serial = device?.serial ?? ''
    form.osFamily = device?.osFamily ?? osFamilyOf(device?.os)
    form.os = device?.os ?? ''
    form.hostId = device?.hostId ? String(device.hostId) : ''
    form.note = device?.note ?? ''
    activeTab.value =
      device && props.initialTab === 'credentials' ? 'credentials' : 'basic'
    const deviceId = device?.id ?? null
    if (credentialsDeviceId.value !== deviceId) {
      credentials.value = []
      credentialsDeviceId.value = deviceId
    }
  },
  { immediate: true },
)

const primaryIp = computed(() => props.device?.ips[0] ?? '')

watch(
  () => form.kind,
  (kind) => {
    if (kind !== 'vm') form.hostId = ''
  },
)

watch([activeTab, () => props.device?.id], ([tab, deviceId]) => {
  if (props.open && tab === 'credentials' && deviceId) {
    void loadCredentials()
  }
})

async function loadCredentials(): Promise<void> {
  if (!props.device) return
  credentialsLoading.value = true
  try {
    credentials.value = await api.listCredentials(props.device.id)
    credentialsDeviceId.value = props.device.id
  } catch (caught) {
    store.notify(caught instanceof Error ? caught.message : '加载凭据失败')
  } finally {
    credentialsLoading.value = false
  }
}

function typeName(typeId: number): string {
  return store.credentialTypes.find((type) => type.id === typeId)?.name ?? '其他'
}

function credentialAddress(credential: Credential): string {
  return credential.address ?? (primaryIp.value || '跟随主 IP')
}

function openAddCredential(): void {
  editingCredential.value = null
  credentialModalOpen.value = true
}

function openEditCredential(credential: Credential): void {
  editingCredential.value = credential
  credentialModalOpen.value = true
}

async function onCredentialSaved(): Promise<void> {
  credentialModalOpen.value = false
  editingCredential.value = null
  await loadCredentials()
}

async function removeCredential(credential: Credential): Promise<void> {
  if (
    !window.confirm(
      `确定删除凭据「${credential.label ?? credential.username ?? '未命名'}」吗？`,
    )
  ) {
    return
  }

  try {
    await api.deleteCredential(credential.id)
    store.notify('凭据已删除')
    await loadCredentials()
    await store.refresh()
  } catch (caught) {
    store.notify(caught instanceof Error ? caught.message : '删除失败')
  }
}

function addIp(): void {
  if (form.ips.length >= 16) return
  form.ips.push('')
}

function removeIp(index: number): void {
  if (form.ips.length <= 1) {
    form.ips[0] = ''
    return
  }
  form.ips.splice(index, 1)
}

function onFamilyChange(): void {
  const options = osOptionsFor(form.osFamily)
  if (!options.includes(form.os)) {
    form.os = options[0] ?? ''
  }
}

async function submit(): Promise<void> {
  if (!form.name.trim()) {
    error.value = '请输入设备名称'
    return
  }
  if (form.rackId && !form.rackPosition) {
    error.value = '放入机柜后需要填写起始 U 位'
    return
  }

  busy.value = true
  error.value = ''
  try {
    const ips = form.ips.map((ip) => ip.trim()).filter(Boolean)
    const payload = {
      name: form.name.trim(),
      kind: form.kind,
      status: form.status,
      rackId: form.rackId ? Number(form.rackId) : null,
      rackPosition: form.rackPosition ? Number(form.rackPosition) : null,
      rackHeight: Number(form.rackHeight) || 1,
      ips,
      vendor: form.vendor.trim() || null,
      model: form.model.trim() || null,
      serial: form.serial.trim() || null,
      os: form.os.trim() || null,
      osFamily: form.os.trim() ? form.osFamily : null,
      hostId: form.kind === 'vm' && form.hostId ? Number(form.hostId) : null,
      note: form.note.trim() || null,
    }
    if (props.device) {
      await store.updateDevice(props.device.id, payload)
      store.notify('设备已更新')
    } else {
      await store.createDevice(payload)
      store.notify('设备已创建')
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
    :title="device ? '编辑设备' : '新增设备'"
    width="max-w-3xl"
    @close="$emit('close')"
  >
    <div class="mb-5 flex gap-1 border-b border-white/10">
      <button
        type="button"
        class="-mb-px border-b-2 px-3 pb-2 text-xs transition"
        :class="
          activeTab === 'basic'
            ? 'border-emerald-400 text-emerald-300'
            : 'border-transparent text-slate-500 hover:text-slate-300'
        "
        @click="activeTab = 'basic'"
      >
        基本信息
      </button>
      <button
        type="button"
        class="-mb-px flex items-center gap-1.5 border-b-2 px-3 pb-2 text-xs transition disabled:cursor-not-allowed disabled:opacity-40"
        :class="
          activeTab === 'credentials'
            ? 'border-emerald-400 text-emerald-300'
            : 'border-transparent text-slate-500 hover:text-slate-300'
        "
        :disabled="!device"
        :title="device ? '管理该设备的凭据' : '保存设备后可添加凭据'"
        @click="activeTab = 'credentials'"
      >
        凭据
        <span
          v-if="credentials.length"
          class="rounded bg-white/5 px-1.5 py-0.5 text-[9px] text-slate-400"
        >
          {{ credentials.length }}
        </span>
      </button>
    </div>

    <form v-show="activeTab === 'basic'" class="space-y-5" @submit.prevent="submit">
      <div class="grid grid-cols-2 gap-4">
        <div>
          <label class="label" for="device-name">设备名称</label>
          <input
            id="device-name"
            v-model="form.name"
            class="input"
            placeholder="如 esxi-01"
            maxlength="60"
          />
        </div>
        <div>
          <label class="label" for="device-kind">类型</label>
          <select id="device-kind" v-model="form.kind" class="input">
            <option v-for="item in kinds" :key="item[0]" :value="item[0]">
              {{ item[1].label }}
            </option>
          </select>
        </div>
      </div>

      <div class="grid grid-cols-3 gap-4">
        <div>
          <label class="label" for="device-rack">所属机柜</label>
          <select id="device-rack" v-model="form.rackId" class="input">
            <option value="">未放入机柜</option>
            <option v-for="rack in store.racks" :key="rack.id" :value="String(rack.id)">
              {{ rack.name }}（{{ rack.totalU }}U）
            </option>
          </select>
        </div>
        <div>
          <label class="label" for="device-position">起始 U 位</label>
          <input
            id="device-position"
            v-model="form.rackPosition"
            class="input"
            type="number"
            min="1"
            max="48"
            placeholder="如 20"
          />
        </div>
        <div>
          <label class="label" for="device-height">占用 U 数</label>
          <input
            id="device-height"
            v-model="form.rackHeight"
            class="input"
            type="number"
            min="1"
            max="48"
          />
        </div>
      </div>

      <div>
        <div class="mb-1.5 flex items-center justify-between">
          <span class="text-xs font-medium text-slate-400">IP 地址</span>
          <button
            type="button"
            class="text-[11px] text-emerald-400 transition hover:text-emerald-300 disabled:opacity-40"
            :disabled="form.ips.length >= 16"
            @click="addIp"
          >
            + 添加 IP
          </button>
        </div>

        <div class="space-y-2">
          <div
            v-for="(_, index) in form.ips"
            :key="index"
            class="flex items-center gap-2"
          >
            <span
              class="w-14 shrink-0 text-[10px]"
              :class="index === 0 ? 'font-medium text-emerald-300' : 'text-slate-500'"
            >
              {{ index === 0 ? '主 IP' : `备用 ${index}` }}
            </span>
            <input
              v-model="form.ips[index]"
              class="input flex-1"
              :placeholder="index === 0 ? '如 10.0.0.21' : '备用 IP（可选）'"
              autocomplete="off"
            />
            <button
              type="button"
              class="rounded-md px-2 py-1.5 text-slate-500 transition hover:bg-white/5 hover:text-slate-300"
              title="移除"
              @click="removeIp(index)"
            >
              ×
            </button>
          </div>
        </div>
        <p class="mt-1.5 text-[10px] text-slate-500">
          第一个地址为主 IP，其余为备用；支持 IPv4 / IPv6，最多 16 个，重复地址会自动合并。
        </p>
      </div>

      <div class="grid grid-cols-2 gap-4">
        <div>
          <label class="label" for="device-vendor">厂商</label>
          <input id="device-vendor" v-model="form.vendor" class="input" />
        </div>
        <div>
          <label class="label" for="device-model">型号</label>
          <input id="device-model" v-model="form.model" class="input" />
        </div>
        <div>
          <label class="label" for="device-serial">序列号</label>
          <input id="device-serial" v-model="form.serial" class="input" />
        </div>
        <div>
          <label class="label" for="device-status">状态</label>
          <select id="device-status" v-model="form.status" class="input">
            <option v-for="status in statuses" :key="status.value" :value="status.value">
              {{ status.label }}
            </option>
          </select>
        </div>
      </div>

      <div>
        <div class="mb-1.5 flex items-center justify-between">
          <span class="text-xs font-medium text-slate-400">操作系统</span>
          <span class="text-[10px] text-slate-600">大类 → 具体版本</span>
        </div>
        <div class="grid grid-cols-2 gap-4">
          <div>
            <label class="sr-only" for="device-os-family">系统大类</label>
            <select
              id="device-os-family"
              v-model="form.osFamily"
              class="input"
              @change="onFamilyChange"
            >
              <option v-for="group in osFamilyGroups" :key="group.value" :value="group.value">
                {{ group.label }}
              </option>
            </select>
          </div>
          <div>
            <label class="sr-only" for="device-os">具体系统</label>
            <select id="device-os" v-model="form.os" class="input">
              <option value="">未指定</option>
              <option v-for="os in currentOsOptions" :key="os" :value="os">
                {{ os }}
              </option>
            </select>
          </div>
        </div>
        <p class="mt-1.5 text-[10px] text-slate-500">
          选择大类后，下方会列出对应的操作系统版本。
        </p>
      </div>

      <div v-if="form.kind === 'vm'">
        <label class="label" for="device-host">宿主机</label>
        <select id="device-host" v-model="form.hostId" class="input">
          <option value="">未指定</option>
          <option v-for="host in hostCandidates" :key="host.id" :value="String(host.id)">
            {{ host.name }}
          </option>
        </select>
        <p class="mt-1.5 text-[10px] text-slate-500">
          宿主机只能是物理服务器；暂不支持虚拟机嵌套虚拟机。
        </p>
      </div>

      <div>
        <label class="label" for="device-note">备注</label>
        <textarea
          id="device-note"
          v-model="form.note"
          class="input resize-none"
          rows="2"
          placeholder="用途、责任人、维保信息…"
        />
      </div>

      <p
        v-if="error"
        class="rounded-lg border border-rose-500/20 bg-rose-500/10 px-3 py-2 text-xs text-rose-300"
      >
        {{ error }}
      </p>
    </form>

    <div v-if="activeTab === 'credentials'" class="space-y-4">
      <div
        v-if="!device"
        class="flex flex-col items-center gap-2 rounded-xl border border-white/5 bg-white/[0.02] px-4 py-10 text-center"
      >
        <KeyRound class="h-6 w-6 text-slate-600" />
        <p class="text-xs text-slate-400">保存设备后即可添加凭据</p>
        <p class="text-[10px] text-slate-600">先填写基本信息并保存，再回到「凭据」标签页。</p>
      </div>

      <template v-else>
        <div class="flex flex-wrap items-center justify-between gap-3">
          <div>
            <p class="text-xs text-slate-300">该设备的凭据</p>
            <p class="mt-0.5 text-[10px] text-slate-500">
              密码只在连接时解密；列表不显示密码。地址留空时跟随设备主 IP。
            </p>
          </div>
          <button class="btn btn-ghost" type="button" @click="openAddCredential">
            <Plus class="h-3.5 w-3.5" />
            添加凭据
          </button>
        </div>

        <p v-if="credentialsLoading" class="py-6 text-center text-[11px] text-slate-500">
          加载凭据…
        </p>

        <div
          v-else-if="!credentials.length"
          class="flex flex-col items-center gap-2 rounded-xl border border-white/5 bg-white/[0.02] px-4 py-10 text-center"
        >
          <KeyRound class="h-6 w-6 text-slate-600" />
          <p class="text-xs text-slate-400">还没有凭据</p>
          <p class="text-[10px] text-slate-600">
            添加 RDP、SSH、iDRAC、ESXi 或 Web 凭据后，设备列表会出现对应的连接按钮。
          </p>
          <button class="btn btn-primary mt-1" type="button" @click="openAddCredential">
            <Plus class="h-3.5 w-3.5" />
            添加第一条
          </button>
        </div>

        <div v-else class="space-y-2">
          <div
            v-for="credential in credentials"
            :key="credential.id"
            class="flex flex-wrap items-center justify-between gap-3 rounded-xl border border-white/5 bg-white/[0.02] px-3 py-2.5"
          >
            <div class="min-w-0">
              <div class="flex flex-wrap items-center gap-2">
                <span class="badge bg-white/5 text-slate-300 ring-white/10">
                  {{ typeName(credential.typeId) }}
                </span>
                <span class="text-xs font-medium text-slate-200">
                  {{ credential.label ?? '未命名' }}
                </span>
                <span
                  v-if="credential.isDefault"
                  class="rounded bg-emerald-500/10 px-1.5 py-0.5 text-[9px] text-emerald-300"
                >
                  默认
                </span>
              </div>
              <p class="mt-1 flex flex-wrap gap-x-3 gap-y-0.5 font-mono text-[10px] text-slate-500">
                <span>{{ credential.username || '无用户名' }}</span>
                <span>{{ credentialAddress(credential) }}</span>
                <span>端口 {{ credential.port ?? '默认' }}</span>
              </p>
            </div>
            <div class="flex shrink-0 gap-1">
              <button
                class="rounded-md p-1.5 text-slate-400 transition hover:bg-white/5 hover:text-slate-200"
                title="编辑凭据"
                type="button"
                @click="openEditCredential(credential)"
              >
                <Pencil class="h-3.5 w-3.5" />
              </button>
              <button
                class="rounded-md p-1.5 text-slate-500 transition hover:bg-rose-500/10 hover:text-rose-300"
                title="删除凭据"
                type="button"
                @click="removeCredential(credential)"
              >
                <Trash2 class="h-3.5 w-3.5" />
              </button>
            </div>
          </div>
        </div>
      </template>
    </div>

    <template #footer>
      <template v-if="activeTab === 'basic'">
        <button class="btn btn-ghost" type="button" @click="$emit('close')">取消</button>
        <button class="btn btn-primary" type="button" :disabled="busy" @click="submit">
          {{ busy ? '保存中…' : device ? '保存修改' : '创建设备' }}
        </button>
      </template>
      <button v-else class="btn btn-ghost" type="button" @click="$emit('close')">
        关闭
      </button>
    </template>
  </Modal>

  <CredentialFormModal
    v-if="device"
    :open="credentialModalOpen"
    :device-id="device.id"
    :primary-ip="primaryIp"
    :credential="editingCredential"
    @close="credentialModalOpen = false"
    @saved="onCredentialSaved"
  />
</template>
