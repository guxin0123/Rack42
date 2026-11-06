<script setup lang="ts">
import {
  Copy,
  Database,
  Download,
  FolderOpen,
  FolderUp,
  Github,
  KeyRound,
  Lock,
  Monitor,
  Moon,
  Palette,
  Pencil,
  Plus,
  Save,
  ShieldCheck,
  Sun,
  Terminal,
  Trash2,
} from 'lucide-vue-next'
import { onMounted, reactive, ref } from 'vue'

import Modal from '@/components/ui/Modal.vue'
import OsFamilySettings from '@/components/settings/OsFamilySettings.vue'
import { iconComponent, iconOptions, isCustomIcon } from '@/lib/icons'
import { api } from '@/lib/ipc'
import type {
  CredentialAction,
  CredentialType,
  CredentialTypeInput,
  SftpClient,
  SshClient,
} from '@/lib/types'
import { useAppStore } from '@/stores/app'
import { useThemeStore } from '@/stores/theme'

const store = useAppStore()
const theme = useThemeStore()

const themeOptions = [
  { value: 'dark', label: '深色', description: '低光环境', icon: Moon },
  { value: 'light', label: '浅色', description: '明亮清晰', icon: Sun },
  { value: 'system', label: '跟随系统', description: '自动切换', icon: Monitor },
] as const

const exporting = ref(false)
const exportedPath = ref('')

const sshClients: Array<{ value: SshClient; label: string }> = [
  { value: 'openssh', label: 'Windows OpenSSH' },
  { value: 'xshell', label: 'Xshell 7.x' },
  { value: 'putty', label: 'PuTTY' },
]

const sftpClients: Array<{ value: SftpClient; label: string }> = [
  { value: 'openssh', label: 'Windows OpenSSH sftp' },
  { value: 'xftp', label: 'Xftp 7.x' },
  { value: 'psftp', label: 'psftp' },
]

const connection = reactive({
  sshClient: 'openssh' as SshClient,
  sshPath: '',
  sftpClient: 'openssh' as SftpClient,
  sftpPath: '',
  autoFillPassword: false,
})

const loadingConnection = ref(true)
const savingConnection = ref(false)
const detectingClient = ref('')

onMounted(() => {
  void loadConnectionSettings()
  void loadCredentialTypes()
})

async function loadConnectionSettings(): Promise<void> {
  loadingConnection.value = true
  try {
    const settings = await api.getConnectionSettings()
    connection.sshClient = settings.sshClient
    connection.sshPath = settings.sshPath ?? ''
    connection.sftpClient = settings.sftpClient
    connection.sftpPath = settings.sftpPath ?? ''
    connection.autoFillPassword = settings.autoFillPassword
  } catch (caught) {
    store.notify(caught instanceof Error ? caught.message : '加载连接设置失败')
  } finally {
    loadingConnection.value = false
  }
}

async function saveConnectionSettings(): Promise<void> {
  savingConnection.value = true
  try {
    await api.updateConnectionSettings({
      sshClient: connection.sshClient,
      sshPath: connection.sshPath.trim() || null,
      sftpClient: connection.sftpClient,
      sftpPath: connection.sftpPath.trim() || null,
      autoFillPassword: connection.autoFillPassword,
    })
    store.notify('远程连接设置已保存')
  } catch (caught) {
    store.notify(caught instanceof Error ? caught.message : '保存连接设置失败')
  } finally {
    savingConnection.value = false
  }
}

async function detectSshClient(): Promise<void> {
  detectingClient.value = 'ssh'
  try {
    const path = await api.detectClientPath(connection.sshClient)
    if (path) {
      connection.sshPath = path
      store.notify('已检测到 SSH 客户端')
    } else {
      store.notify('未自动检测到 SSH 客户端，请手动填写路径')
    }
  } catch (caught) {
    store.notify(caught instanceof Error ? caught.message : '检测客户端失败')
  } finally {
    detectingClient.value = ''
  }
}

async function detectSftpClient(): Promise<void> {
  detectingClient.value = 'sftp'
  try {
    const client =
      connection.sftpClient === 'openssh' ? 'openssh-sftp' : connection.sftpClient
    const path = await api.detectClientPath(client)
    if (path) {
      connection.sftpPath = path
      store.notify('已检测到 SFTP 客户端')
    } else {
      store.notify('未自动检测到 SFTP 客户端，请手动填写路径')
    }
  } catch (caught) {
    store.notify(caught instanceof Error ? caught.message : '检测客户端失败')
  } finally {
    detectingClient.value = ''
  }
}

const credentialTypes = ref<CredentialType[]>([])
const typeLoading = ref(true)
const typeModalOpen = ref(false)
const editingType = ref<CredentialType | null>(null)
const typeSaving = ref(false)
const typeError = ref('')

const typeForm = reactive({
  name: '',
  icon: 'key',
  actions: [] as CredentialAction[],
  defaultPort: '',
  sortOrder: '0',
})

const actionOptions: Array<{ value: CredentialAction; label: string }> = [
  { value: 'rdp', label: 'RDP' },
  { value: 'ssh', label: 'SSH' },
  { value: 'sftp', label: 'SFTP' },
  { value: 'web', label: 'Web' },
]

const typeIcon = iconComponent

function onIconFile(event: Event): void {
  const input = event.target as HTMLInputElement
  const file = input.files?.[0]
  input.value = ''
  if (!file) return

  if (!['image/png', 'image/jpeg', 'image/webp'].includes(file.type)) {
    typeError.value = '自定义图标只支持 PNG / JPEG / WebP'
    return
  }
  if (file.size > 100 * 1024) {
    typeError.value = '图标文件不能超过 100KB'
    return
  }

  const reader = new FileReader()
  reader.onload = () => {
    if (typeof reader.result === 'string') {
      typeForm.icon = reader.result
      typeError.value = ''
    }
  }
  reader.onerror = () => {
    typeError.value = '读取图标失败'
  }
  reader.readAsDataURL(file)
}

async function loadCredentialTypes(): Promise<void> {
  typeLoading.value = true
  try {
    credentialTypes.value = await api.listCredentialTypes()
  } catch (caught) {
    store.notify(caught instanceof Error ? caught.message : '加载凭据类型失败')
  } finally {
    typeLoading.value = false
  }
}

function openCreateType(): void {
  editingType.value = null
  typeForm.name = ''
  typeForm.icon = 'key'
  typeForm.actions = []
  typeForm.defaultPort = ''
  typeForm.sortOrder = '100'
  typeError.value = ''
  typeModalOpen.value = true
}

function openEditType(type: CredentialType): void {
  editingType.value = type
  typeForm.name = type.name
  typeForm.icon = type.icon
  typeForm.actions = [...type.actions]
  typeForm.defaultPort = type.defaultPort ? String(type.defaultPort) : ''
  typeForm.sortOrder = String(type.sortOrder)
  typeError.value = ''
  typeModalOpen.value = true
}

function toggleTypeAction(action: CredentialAction): void {
  const index = typeForm.actions.indexOf(action)
  if (index >= 0) typeForm.actions.splice(index, 1)
  else typeForm.actions.push(action)
}

function actionName(action: string): string {
  return actionOptions.find((option) => option.value === action)?.label ?? action
}

async function saveType(): Promise<void> {
  if (!typeForm.name.trim()) {
    typeError.value = '请输入类型名称'
    return
  }

  const port = typeForm.defaultPort ? Number(typeForm.defaultPort) : null
  if (port !== null && (!Number.isInteger(port) || port < 1 || port > 65535)) {
    typeError.value = '默认端口需要在 1 到 65535 之间'
    return
  }

  const input: CredentialTypeInput = {
    name: typeForm.name.trim(),
    icon: typeForm.icon,
    actions: [...typeForm.actions],
    defaultPort: port,
    sortOrder: Number(typeForm.sortOrder) || 0,
  }

  typeSaving.value = true
  typeError.value = ''
  try {
    if (editingType.value) {
      await api.updateCredentialType(editingType.value.id, input)
      store.notify('凭据类型已更新')
    } else {
      await api.createCredentialType(input)
      store.notify('凭据类型已创建')
    }
    await loadCredentialTypes()
    await store.refresh()
    typeModalOpen.value = false
  } catch (caught) {
    typeError.value = caught instanceof Error ? caught.message : '保存失败'
  } finally {
    typeSaving.value = false
  }
}

async function removeType(type: CredentialType): Promise<void> {
  if (type.isBuiltin) return
  if (!window.confirm(`确定删除凭据类型「${type.name}」吗？关联凭据会归入「其他」。`)) {
    return
  }

  try {
    await api.deleteCredentialType(type.id)
    store.notify('凭据类型已删除')
    await loadCredentialTypes()
    await store.refresh()
  } catch (caught) {
    store.notify(caught instanceof Error ? caught.message : '删除失败')
  }
}

async function lockNow(): Promise<void> {
  try {
    await store.lock()
  } catch (caught) {
    store.notify(caught instanceof Error ? caught.message : '锁定失败')
  }
}

async function copyDataDir(): Promise<void> {
  try {
    await navigator.clipboard.writeText(store.dataDir)
    store.notify('数据目录已复制')
  } catch {
    store.notify('复制失败，请手动选择路径')
  }
}

async function exportData(): Promise<void> {
  exporting.value = true
  exportedPath.value = ''
  try {
    exportedPath.value = await api.exportData()
    store.notify('导出完成（已脱敏，不含密码）')
  } catch (caught) {
    store.notify(caught instanceof Error ? caught.message : '导出失败')
  } finally {
    exporting.value = false
  }
}
</script>

<template>
  <div class="mx-auto max-w-4xl space-y-5">
    <div>
      <h2 class="text-base font-semibold tracking-tight text-slate-100">设置</h2>
      <p class="mt-1 text-xs text-slate-500">安全状态、主题、远程连接、数据目录与导出</p>
    </div>

    <section class="card p-5">
      <div class="flex items-start justify-between gap-4">
        <div class="flex items-start gap-3">
          <div
            class="flex h-9 w-9 items-center justify-center rounded-lg bg-emerald-500/10 text-emerald-300 ring-1 ring-emerald-500/20"
          >
            <ShieldCheck class="h-4 w-4" />
          </div>
          <div>
            <h3 class="text-xs font-semibold tracking-wide text-slate-200">安全</h3>
            <p class="mt-1 text-[11px] leading-5 text-slate-500">
              主密码只在本机内存中使用，用于派生凭据加密密钥，不会写入磁盘。
            </p>
          </div>
        </div>
        <button class="btn btn-ghost" @click="lockNow">
          <Lock class="h-3.5 w-3.5" />
          立即锁定
        </button>
      </div>

      <dl class="mt-5 grid grid-cols-2 gap-4 border-t border-white/5 pt-4 text-[11px]">
        <div>
          <dt class="text-slate-500">主密码</dt>
          <dd class="mt-1 text-slate-300">已设置 · 每次启动需解锁</dd>
        </div>
        <div>
          <dt class="text-slate-500">当前会话</dt>
          <dd class="mt-1 flex items-center gap-1.5 text-emerald-300">
            <span class="h-1.5 w-1.5 rounded-full bg-emerald-400" />
            已解锁
          </dd>
        </div>
        <div>
          <dt class="text-slate-500">密钥派生</dt>
          <dd class="mt-1 font-mono text-slate-300">Argon2id · 64 MiB · t=3 · p=4</dd>
        </div>
        <div>
          <dt class="text-slate-500">凭据加密</dt>
          <dd class="mt-1 font-mono text-slate-300">XChaCha20-Poly1305</dd>
        </div>
      </dl>
    </section>

    <section class="card p-5">
      <div class="flex items-center gap-3">
        <div
          class="flex h-9 w-9 items-center justify-center rounded-lg bg-violet-500/10 text-violet-300 ring-1 ring-violet-500/20"
        >
          <Palette class="h-4 w-4" />
        </div>
        <div>
          <h3 class="text-xs font-semibold tracking-wide text-slate-200">外观</h3>
          <p class="mt-1 text-[11px] text-slate-500">
            选择界面主题；跟随系统时会随 Windows 的深浅色设置自动切换。
          </p>
        </div>
      </div>

      <div class="mt-4 grid grid-cols-3 gap-3">
        <button
          v-for="option in themeOptions"
          :key="option.value"
          type="button"
          class="rounded-xl border p-3 text-left transition"
          :class="
            theme.mode === option.value
              ? 'border-emerald-400/50 bg-emerald-400/10'
              : 'border-white/10 bg-white/[0.02] hover:border-white/20 hover:bg-white/5'
          "
          @click="theme.setMode(option.value)"
        >
          <component
            :is="option.icon"
            class="h-4 w-4"
            :class="theme.mode === option.value ? 'text-emerald-300' : 'text-slate-400'"
          />
          <p class="mt-2 text-xs font-medium text-slate-200">{{ option.label }}</p>
          <p class="mt-0.5 text-[10px] text-slate-500">{{ option.description }}</p>
        </button>
      </div>
    </section>

    <section class="card p-5">
      <div class="flex items-start gap-3">
        <div
          class="flex h-9 w-9 items-center justify-center rounded-lg bg-emerald-500/10 text-emerald-300 ring-1 ring-emerald-500/20"
        >
          <Terminal class="h-4 w-4" />
        </div>
        <div>
          <h3 class="text-xs font-semibold tracking-wide text-slate-200">远程连接</h3>
          <p class="mt-1 text-[11px] leading-5 text-slate-500">
            SSH/SFTP 默认只传主机、端口和用户名，密码由客户端询问；自动填充默认关闭。
          </p>
        </div>
      </div>

      <div v-if="loadingConnection" class="mt-4 text-[11px] text-slate-500">
        加载连接设置…
      </div>

      <div v-else class="mt-4 space-y-4">
        <div class="grid grid-cols-1 gap-3 md:grid-cols-3">
          <div>
            <label class="label" for="ssh-client">SSH 客户端</label>
            <select id="ssh-client" v-model="connection.sshClient" class="input">
              <option v-for="option in sshClients" :key="option.value" :value="option.value">
                {{ option.label }}
              </option>
            </select>
          </div>
          <div class="md:col-span-2">
            <label class="label" for="ssh-path">SSH 客户端路径</label>
            <div class="flex gap-2">
              <input
                id="ssh-path"
                v-model="connection.sshPath"
                class="input min-w-0 flex-1"
                placeholder="留空自动检测，例如 C:\Program Files\PuTTY\putty.exe"
                spellcheck="false"
              />
              <button
                class="btn btn-ghost shrink-0"
                :disabled="detectingClient === 'ssh'"
                @click="detectSshClient"
              >
                <FolderOpen class="h-3.5 w-3.5" />
                {{ detectingClient === 'ssh' ? '检测中…' : '检测' }}
              </button>
            </div>
          </div>
        </div>

        <div class="grid grid-cols-1 gap-3 md:grid-cols-3">
          <div>
            <label class="label flex items-center gap-1.5" for="sftp-client">
              <FolderUp class="h-3 w-3" />
              SFTP 客户端
            </label>
            <select id="sftp-client" v-model="connection.sftpClient" class="input">
              <option v-for="option in sftpClients" :key="option.value" :value="option.value">
                {{ option.label }}
              </option>
            </select>
          </div>
          <div class="md:col-span-2">
            <label class="label" for="sftp-path">SFTP 客户端路径</label>
            <div class="flex gap-2">
              <input
                id="sftp-path"
                v-model="connection.sftpPath"
                class="input min-w-0 flex-1"
                placeholder="留空自动检测，例如 C:\Program Files\PuTTY\psftp.exe"
                spellcheck="false"
              />
              <button
                class="btn btn-ghost shrink-0"
                :disabled="detectingClient === 'sftp'"
                @click="detectSftpClient"
              >
                <FolderOpen class="h-3.5 w-3.5" />
                {{ detectingClient === 'sftp' ? '检测中…' : '检测' }}
              </button>
            </div>
          </div>
        </div>

        <label class="flex items-start gap-2 text-[11px] text-slate-400">
          <input
            v-model="connection.autoFillPassword"
            type="checkbox"
            class="mt-0.5 accent-emerald-500"
          />
          <span>
            自动填充密码到命令行
            <span class="block text-[10px] text-slate-600">
              默认关闭。Windows OpenSSH 不支持此方式，始终由控制台提示密码。
            </span>
          </span>
        </label>

        <p
          v-if="connection.autoFillPassword"
          class="rounded-lg border border-rose-500/20 bg-rose-500/10 px-3 py-2 text-[11px] leading-5 text-rose-300"
        >
          密码会出现在 Xshell / PuTTY / Xftp / psftp 的命令行中，可能被其他进程或审计工具读取。
          仅建议在隔离测试环境使用；日常使用请保持关闭，由客户端弹窗输入。
        </p>

        <p class="text-[10px] text-slate-600">
          绿色版 Xshell/Xftp 不在 PATH 时需手动填写可执行文件路径；保存时会校验文件名和是否存在。
        </p>

        <div class="flex justify-end border-t border-white/5 pt-4">
          <button class="btn btn-primary" :disabled="savingConnection" @click="saveConnectionSettings">
            <Save class="h-3.5 w-3.5" />
            {{ savingConnection ? '保存中…' : '保存连接设置' }}
          </button>
        </div>
      </div>
    </section>

    <section class="card p-5">
      <div class="flex flex-wrap items-start justify-between gap-3">
        <div class="flex items-start gap-3">
          <div
            class="flex h-9 w-9 items-center justify-center rounded-lg bg-amber-500/10 text-amber-300 ring-1 ring-amber-500/20"
          >
            <KeyRound class="h-4 w-4" />
          </div>
          <div>
            <h3 class="text-xs font-semibold tracking-wide text-slate-200">凭据类型</h3>
            <p class="mt-1 text-[11px] leading-5 text-slate-500">
              自定义类型可设置名称、图标、动作和默认端口；内置类型不可删除。
            </p>
          </div>
        </div>
        <button class="btn btn-ghost" @click="openCreateType">
          <Plus class="h-3.5 w-3.5" />
          新增类型
        </button>
      </div>

      <p v-if="typeLoading" class="mt-4 text-[11px] text-slate-500">加载类型…</p>

      <div v-else class="mt-4 overflow-x-auto">
        <table class="w-full min-w-[36rem] text-left text-[11px]">
          <thead class="text-[10px] uppercase tracking-wider text-slate-500">
            <tr class="border-b border-white/5">
              <th class="py-2 pr-4 font-medium">类型</th>
              <th class="px-2 py-2 font-medium">动作</th>
              <th class="px-2 py-2 font-medium">默认端口</th>
              <th class="py-2 pl-4 text-right font-medium">操作</th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="type in credentialTypes"
              :key="type.id"
              class="border-b border-white/[0.03] last:border-0"
            >
              <td class="py-2.5 pr-4">
                <div class="flex items-center gap-2">
                  <img
                    v-if="isCustomIcon(type.icon)"
                    :src="type.icon"
                    alt=""
                    class="h-4 w-4 rounded object-cover"
                  />
                  <component
                    :is="typeIcon(type.icon)"
                    v-else
                    class="h-3.5 w-3.5 text-slate-400"
                  />
                  <span class="font-medium text-slate-200">{{ type.name }}</span>
                  <span
                    v-if="type.isBuiltin"
                    class="rounded bg-white/5 px-1.5 py-0.5 text-[9px] text-slate-500"
                  >
                    内置
                  </span>
                </div>
              </td>
              <td class="px-2 py-2.5">
                <div class="flex flex-wrap gap-1">
                  <span
                    v-for="action in type.actions"
                    :key="action"
                    class="rounded bg-white/5 px-1.5 py-0.5 text-[9px] text-slate-400"
                  >
                    {{ actionName(action) }}
                  </span>
                  <span v-if="!type.actions.length" class="text-[10px] text-slate-600">无</span>
                </div>
              </td>
              <td class="px-2 py-2.5 font-mono text-slate-400">
                {{ type.defaultPort ?? '—' }}
              </td>
              <td class="py-2.5 pl-4">
                <div class="flex justify-end gap-1">
                  <button
                    class="rounded-md p-1.5 text-slate-400 transition hover:bg-white/5 hover:text-slate-200"
                    title="编辑类型"
                    @click="openEditType(type)"
                  >
                    <Pencil class="h-3.5 w-3.5" />
                  </button>
                  <button
                    class="rounded-md p-1.5 text-slate-500 transition hover:bg-rose-500/10 hover:text-rose-300 disabled:cursor-not-allowed disabled:opacity-30"
                    title="删除类型"
                    :disabled="type.isBuiltin"
                    @click="removeType(type)"
                  >
                    <Trash2 class="h-3.5 w-3.5" />
                  </button>
                </div>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </section>

    <OsFamilySettings />

    <section class="card p-5">
      <div class="flex items-center gap-3">
        <div
          class="flex h-9 w-9 items-center justify-center rounded-lg bg-sky-500/10 text-sky-300 ring-1 ring-sky-500/20"
        >
          <Database class="h-4 w-4" />
        </div>
        <div>
          <h3 class="text-xs font-semibold tracking-wide text-slate-200">数据</h3>
          <p class="mt-1 text-[11px] text-slate-500">
            数据库与导出文件都保存在本机，不依赖任何云端服务。
          </p>
        </div>
      </div>

      <div class="mt-4 flex items-center justify-between text-[11px]">
        <span class="text-slate-500">存储模式</span>
        <span class="text-slate-300">
          {{ store.portable ? '便携模式（程序目录 data）' : '标准模式（用户数据目录）' }}
        </span>
      </div>

      <div class="mt-3 flex items-center gap-2 rounded-lg border border-white/5 bg-slate-950/60 px-3 py-2">
        <FolderOpen class="h-3.5 w-3.5 shrink-0 text-slate-500" />
        <code class="min-w-0 flex-1 truncate text-[11px] text-slate-400">
          {{ store.dataDir }}
        </code>
        <button
          class="rounded p-1 text-slate-500 transition hover:bg-white/5 hover:text-slate-300"
          title="复制路径"
          @click="copyDataDir"
        >
          <Copy class="h-3.5 w-3.5" />
        </button>
      </div>

      <div class="mt-4 flex items-center justify-between gap-4 border-t border-white/5 pt-4">
        <div>
          <p class="text-[11px] text-slate-400">导出资产数据（JSON）</p>
          <p class="mt-0.5 text-[10px] text-slate-600">
            默认脱敏，不包含密码、密钥与主密码哈希。
          </p>
        </div>
        <button class="btn btn-ghost" :disabled="exporting" @click="exportData">
          <Download class="h-3.5 w-3.5" />
          {{ exporting ? '导出中…' : '导出 JSON' }}
        </button>
      </div>

      <p
        v-if="exportedPath"
        class="mt-3 break-all rounded-lg border border-emerald-500/20 bg-emerald-500/5 px-3 py-2 font-mono text-[10px] text-emerald-300"
      >
        {{ exportedPath }}
      </p>
    </section>

    <section class="card p-5">
      <h3 class="text-xs font-semibold tracking-wide text-slate-200">关于 Rack42</h3>
      <p class="mt-2 max-w-2xl text-[11px] leading-5 text-slate-500">
        面向中小机房的本地优先机架与基础设施管理工具：机架 U 位可视化、设备台账、
        连接拓扑与加密凭据。数据完全保存在本机。
      </p>

      <div class="mt-4 flex flex-wrap gap-2">
        <span
          v-for="item in ['Tauri 2', 'Vue 3.5', 'TypeScript', 'Pinia', 'Vue Flow', 'SQLite', 'Argon2id']"
          :key="item"
          class="rounded-md border border-white/5 bg-white/[0.03] px-2 py-1 text-[10px] text-slate-400"
        >
          {{ item }}
        </span>
      </div>

      <div class="mt-5 flex items-center justify-between border-t border-white/5 pt-4">
        <span class="flex items-center gap-1.5 text-[11px] text-slate-400">
          <Github class="h-3.5 w-3.5" />
          github.com/guxin0123/Rack42
        </span>
        <span class="text-[10px] text-slate-600">v1.0.0 · MIT</span>
      </div>
    </section>

    <Modal
      :open="typeModalOpen"
      :title="editingType ? '编辑凭据类型' : '新增凭据类型'"
      width="max-w-lg"
      @close="typeModalOpen = false"
    >
      <form class="space-y-4" @submit.prevent="saveType">
        <div>
          <label class="label" for="type-name">名称</label>
          <input
            id="type-name"
            v-model="typeForm.name"
            class="input"
            placeholder="如 数据库 / VPN"
            maxlength="40"
          />
        </div>

        <div>
          <label class="label">图标</label>
          <div class="grid grid-cols-5 gap-2">
            <button
              v-for="option in iconOptions"
              :key="option.value"
              type="button"
              class="flex flex-col items-center gap-1 rounded-lg border px-2 py-2 transition"
              :class="
                typeForm.icon === option.value
                  ? 'border-emerald-400/50 bg-emerald-400/10 text-emerald-300'
                  : 'border-white/10 bg-white/[0.02] text-slate-400 hover:border-white/20'
              "
              @click="typeForm.icon = option.value"
            >
              <component :is="option.icon" class="h-4 w-4" />
              <span class="text-[9px]">{{ option.label }}</span>
            </button>
          </div>

          <div class="mt-2 flex flex-wrap items-center gap-2">
            <label class="btn btn-ghost cursor-pointer">
              <Plus class="h-3.5 w-3.5" />
              上传自定义图标
              <input
                type="file"
                accept="image/png,image/jpeg,image/webp"
                class="hidden"
                @change="onIconFile"
              />
            </label>
            <div
              v-if="isCustomIcon(typeForm.icon)"
              class="flex items-center gap-2 rounded-lg border border-white/10 bg-white/[0.02] px-2 py-1"
            >
              <img :src="typeForm.icon" alt="" class="h-6 w-6 rounded object-cover" />
              <span class="text-[10px] text-slate-400">自定义</span>
              <button
                type="button"
                class="text-[10px] text-slate-500 hover:text-rose-300"
                @click="typeForm.icon = 'key'"
              >
                移除
              </button>
            </div>
          </div>
          <p class="mt-1.5 text-[10px] text-slate-600">
            PNG / JPEG / WebP，最大 100KB；不支持 SVG。
          </p>
        </div>

        <div>
          <label class="label">动作</label>
          <div class="flex flex-wrap gap-4">
            <label
              v-for="action in actionOptions"
              :key="action.value"
              class="flex items-center gap-2 text-[11px] text-slate-400"
            >
              <input
                type="checkbox"
                class="accent-emerald-500"
                :checked="typeForm.actions.includes(action.value)"
                @change="toggleTypeAction(action.value)"
              />
              {{ action.label }}
            </label>
          </div>
          <p class="mt-1.5 text-[10px] text-slate-600">
            连接按钮会按这里的动作显示；允许为空。
          </p>
        </div>

        <div class="grid grid-cols-2 gap-4">
          <div>
            <label class="label" for="type-port">默认端口</label>
            <input
              id="type-port"
              v-model="typeForm.defaultPort"
              class="input"
              type="number"
              min="1"
              max="65535"
              placeholder="留空表示无"
            />
          </div>
          <div>
            <label class="label" for="type-sort">排序</label>
            <input
              id="type-sort"
              v-model="typeForm.sortOrder"
              class="input"
              type="number"
            />
          </div>
        </div>

        <p v-if="editingType?.isBuiltin" class="text-[10px] text-slate-500">
          这是内置类型，可以修改展示信息，但不能删除。
        </p>

        <p
          v-if="typeError"
          class="rounded-lg border border-rose-500/20 bg-rose-500/10 px-3 py-2 text-xs text-rose-300"
        >
          {{ typeError }}
        </p>
      </form>

      <template #footer>
        <button class="btn btn-ghost" @click="typeModalOpen = false">取消</button>
        <button class="btn btn-primary" :disabled="typeSaving" @click="saveType">
          {{ typeSaving ? '保存中…' : '保存类型' }}
        </button>
      </template>
    </Modal>
  </div>
</template>
