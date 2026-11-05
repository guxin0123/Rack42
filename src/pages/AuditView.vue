<script setup lang="ts">
import { RefreshCw, ScrollText, Trash2 } from 'lucide-vue-next'
import { onMounted, ref } from 'vue'

import { formatDate } from '@/lib/format'
import { api } from '@/lib/ipc'
import type { AuditLog } from '@/lib/types'
import { useAppStore } from '@/stores/app'

const store = useAppStore()
const logs = ref<AuditLog[]>([])
const loading = ref(true)

const actionLabels: Record<string, string> = {
  rdp: 'RDP',
  web: 'Web',
  ssh: 'SSH',
  sftp: 'SFTP',
}

onMounted(() => {
  void loadLogs()
})

async function loadLogs(): Promise<void> {
  loading.value = true
  try {
    logs.value = await api.listAuditLogs(200)
  } catch (caught) {
    store.notify(caught instanceof Error ? caught.message : '加载审计日志失败')
  } finally {
    loading.value = false
  }
}

async function clearLogs(): Promise<void> {
  if (!window.confirm('确定清空全部审计日志吗？该操作不可撤销。')) return

  try {
    await api.clearAuditLogs()
    logs.value = []
    store.notify('审计日志已清空')
  } catch (caught) {
    store.notify(caught instanceof Error ? caught.message : '清空失败')
  }
}
</script>

<template>
  <div class="space-y-5">
    <div class="flex flex-wrap items-end justify-between gap-3">
      <div>
        <h2 class="text-base font-semibold tracking-tight text-slate-100">审计日志</h2>
        <p class="mt-1 text-xs text-slate-500">
          只记录连接动作、目标与结果；不记录密码或用户名，保留最近 200 条。
        </p>
      </div>
      <div class="flex items-center gap-2">
        <button class="btn btn-ghost" :disabled="loading" @click="loadLogs">
          <RefreshCw class="h-3.5 w-3.5" :class="{ 'animate-spin': loading }" />
          刷新
        </button>
        <button
          class="btn btn-ghost text-rose-300"
          :disabled="!logs.length"
          @click="clearLogs"
        >
          <Trash2 class="h-3.5 w-3.5" />
          清空
        </button>
      </div>
    </div>

    <div class="card overflow-hidden">
      <div class="flex items-center justify-between border-b border-white/5 px-5 py-3.5">
        <p class="flex items-center gap-2 text-[11px] text-slate-500">
          <ScrollText class="h-3.5 w-3.5" />
          {{ loading ? '加载中…' : `${logs.length} 条记录` }}
        </p>
        <p class="text-[10px] text-slate-600">成功与失败都会记录</p>
      </div>

      <div v-if="logs.length" class="overflow-x-auto">
        <table class="w-full min-w-[58rem] text-left text-xs">
          <thead class="text-[10px] uppercase tracking-wider text-slate-500">
            <tr class="border-b border-white/5">
              <th class="px-5 py-3 font-medium">时间</th>
              <th class="px-5 py-3 font-medium">动作</th>
              <th class="px-5 py-3 font-medium">设备</th>
              <th class="px-5 py-3 font-medium">凭据</th>
              <th class="px-5 py-3 font-medium">目标</th>
              <th class="px-5 py-3 font-medium">结果</th>
              <th class="px-5 py-3 font-medium">详情</th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="log in logs"
              :key="log.id"
              class="border-b border-white/[0.03] last:border-0 hover:bg-white/[0.02]"
            >
              <td class="whitespace-nowrap px-5 py-3 font-mono text-[10px] text-slate-500">
                {{ formatDate(log.createdAt) }}
              </td>
              <td class="px-5 py-3">
                <span class="badge bg-white/5 text-slate-300 ring-white/10">
                  {{ actionLabels[log.action] ?? log.action }}
                </span>
              </td>
              <td class="px-5 py-3 text-slate-300">{{ log.deviceName ?? '—' }}</td>
              <td class="px-5 py-3">
                <span class="text-slate-300">{{ log.credentialLabel ?? '未命名' }}</span>
                <span v-if="log.credentialType" class="ml-1 text-[10px] text-slate-500">
                  · {{ log.credentialType }}
                </span>
              </td>
              <td class="px-5 py-3 font-mono text-[10px] text-slate-400">
                {{ log.target ?? '—' }}
              </td>
              <td class="px-5 py-3">
                <span
                  class="rounded px-1.5 py-0.5 text-[10px]"
                  :class="
                    log.result === 'success'
                      ? 'bg-emerald-500/10 text-emerald-300'
                      : 'bg-rose-500/10 text-rose-300'
                  "
                >
                  {{ log.result === 'success' ? '成功' : '失败' }}
                </span>
              </td>
              <td
                class="max-w-72 truncate px-5 py-3 text-[10px] text-slate-500"
                :title="log.detail ?? ''"
              >
                {{ log.detail ?? '—' }}
              </td>
            </tr>
          </tbody>
        </table>
      </div>

      <div v-else class="flex flex-col items-center gap-2 px-6 py-16 text-center">
        <ScrollText class="h-7 w-7 text-slate-600" />
        <p class="text-xs text-slate-400">还没有连接记录</p>
        <p class="max-w-sm text-[10px] leading-5 text-slate-600">
          从设备列表发起 RDP、Web、SSH 或 SFTP 连接后，这里会出现成功/失败记录。
        </p>
      </div>
    </div>
  </div>
</template>
