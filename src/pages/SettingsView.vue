<script setup lang="ts">
import {
  Copy,
  Database,
  Download,
  FolderOpen,
  Github,
  Lock,
  Monitor,
  Moon,
  Palette,
  ShieldCheck,
  Sun,
} from 'lucide-vue-next'
import { ref } from 'vue'

import { api } from '@/lib/ipc'
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
      <p class="mt-1 text-xs text-slate-500">安全状态、主题、数据目录与导出</p>
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
        <span class="text-[10px] text-slate-600">v0.1.0 · MIT</span>
      </div>
    </section>
  </div>
</template>
