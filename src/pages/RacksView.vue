<script setup lang="ts">
import { MapPin, Pencil, Plus, Server, Trash2 } from 'lucide-vue-next'
import { computed, ref } from 'vue'
import { useRouter } from 'vue-router'

import RackFormModal from '@/components/rack/RackFormModal.vue'
import type { Rack } from '@/lib/types'
import { useAppStore } from '@/stores/app'

const store = useAppStore()
const router = useRouter()

const formOpen = ref(false)
const editing = ref<Rack | null>(null)
const busyId = ref<number | null>(null)

const rows = computed(() =>
  store.racks
    .map((rack) => {
      const devices = store.devices.filter((device) => device.rackId === rack.id)
      const used = devices
        .filter((device) => device.rackPosition)
        .reduce((sum, device) => sum + device.rackHeight, 0)
      return {
        rack,
        deviceCount: devices.length,
        used,
        pct: rack.totalU > 0 ? Math.min(100, Math.round((used / rack.totalU) * 100)) : 0,
      }
    })
    .sort((a, b) => a.rack.name.localeCompare(b.rack.name)),
)

function openCreate(): void {
  editing.value = null
  formOpen.value = true
}

function openEdit(rack: Rack): void {
  editing.value = rack
  formOpen.value = true
}

async function removeRack(rack: Rack): Promise<void> {
  const confirmed = window.confirm(
    `确定删除机柜「${rack.name}」吗？机柜内设备会保留，但会解除机柜与 U 位绑定。`,
  )
  if (!confirmed) return

  busyId.value = rack.id
  try {
    await store.deleteRack(rack.id)
    store.notify('机柜已删除')
  } catch (caught) {
    store.notify(caught instanceof Error ? caught.message : '删除失败')
  } finally {
    busyId.value = null
  }
}
</script>

<template>
  <div class="space-y-5">
    <div class="flex items-end justify-between">
      <div>
        <h2 class="text-base font-semibold tracking-tight text-slate-100">机柜</h2>
        <p class="mt-1 text-xs text-slate-500">
          {{ store.racks.length }} 个机柜 · 点击卡片查看 U 位视图
        </p>
      </div>
      <button class="btn btn-primary" @click="openCreate">
        <Plus class="h-3.5 w-3.5" />
        新建机柜
      </button>
    </div>

    <div v-if="rows.length" class="grid grid-cols-3 gap-4">
      <article
        v-for="row in rows"
        :key="row.rack.id"
        class="card group cursor-pointer p-4 transition hover:border-emerald-400/30 hover:bg-slate-900"
        @click="router.push(`/racks/${row.rack.id}`)"
      >
        <div class="flex items-start justify-between gap-3">
          <div class="min-w-0">
            <p class="truncate text-sm font-semibold text-slate-100">
              {{ row.rack.name }}
            </p>
            <p class="mt-1 flex items-center gap-1 truncate text-[11px] text-slate-500">
              <MapPin class="h-3 w-3 shrink-0" />
              {{ row.rack.location ?? '未填写位置' }}
            </p>
          </div>
          <div
            class="flex gap-1 opacity-0 transition group-hover:opacity-100"
            @click.stop
          >
            <button
              class="rounded-md p-1.5 text-slate-400 transition hover:bg-white/10 hover:text-slate-200"
              title="编辑"
              @click="openEdit(row.rack)"
            >
              <Pencil class="h-3.5 w-3.5" />
            </button>
            <button
              class="rounded-md p-1.5 text-slate-500 transition hover:bg-rose-500/10 hover:text-rose-300"
              title="删除"
              :disabled="busyId === row.rack.id"
              @click="removeRack(row.rack)"
            >
              <Trash2 class="h-3.5 w-3.5" />
            </button>
          </div>
        </div>

        <div class="mt-5 flex items-end justify-between">
          <p class="text-2xl font-semibold tracking-tight text-slate-100">
            {{ row.used
            }}<span class="ml-0.5 text-xs font-normal text-slate-500">
              / {{ row.rack.totalU }}U</span
            >
          </p>
          <p class="text-[11px] text-slate-500">{{ row.deviceCount }} 台设备</p>
        </div>

        <div class="mt-2 h-1.5 overflow-hidden rounded-full bg-white/5">
          <div
            class="h-full rounded-full transition-all"
            :class="
              row.pct > 85 ? 'bg-rose-400' : row.pct > 60 ? 'bg-amber-400' : 'bg-emerald-400'
            "
            :style="{ width: `${row.pct}%` }"
          />
        </div>
      </article>
    </div>

    <div
      v-else
      class="card flex flex-col items-center justify-center gap-3 px-6 py-16 text-center"
    >
      <div
        class="flex h-12 w-12 items-center justify-center rounded-xl bg-emerald-500/10 text-emerald-300 ring-1 ring-emerald-500/20"
      >
        <Server class="h-5 w-5" />
      </div>
      <div>
        <p class="text-sm font-medium text-slate-200">还没有机柜</p>
        <p class="mt-1 text-xs text-slate-500">
          创建机柜后即可在 U 位视图中放置服务器、交换机等设备。
        </p>
      </div>
      <button class="btn btn-primary mt-1" @click="openCreate">
        <Plus class="h-3.5 w-3.5" />
        创建第一个机柜
      </button>
    </div>

    <RackFormModal
      :open="formOpen"
      :rack="editing"
      @close="formOpen = false"
      @saved="formOpen = false"
    />
  </div>
</template>
