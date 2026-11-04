<script setup lang="ts">
import { ArrowLeft, MapPin, Pencil, Plus, Trash2 } from 'lucide-vue-next'
import { computed, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'

import DeviceFormModal from '@/components/device/DeviceFormModal.vue'
import RackFormModal from '@/components/rack/RackFormModal.vue'
import RackGrid from '@/components/rack/RackGrid.vue'
import {
  deviceKindLabel,
  deviceStatusLabel,
} from '@/lib/format'
import type { Device } from '@/lib/types'
import { useAppStore } from '@/stores/app'

const store = useAppStore()
const route = useRoute()
const router = useRouter()

const rackId = computed(() => Number(route.params.id))
const rack = computed(() => store.racks.find((item) => item.id === rackId.value) ?? null)

const devices = computed(() =>
  store.devices
    .filter((device) => device.rackId === rackId.value)
    .sort((a, b) => (b.rackPosition ?? 0) - (a.rackPosition ?? 0)),
)

const usedU = computed(() =>
  devices.value
    .filter((device) => device.rackPosition)
    .reduce((sum, device) => sum + device.rackHeight, 0),
)

const pct = computed(() =>
  rack.value && rack.value.totalU > 0
    ? Math.round((usedU.value / rack.value.totalU) * 100)
    : 0,
)

const rackFormOpen = ref(false)
const deviceFormOpen = ref(false)
const editingDevice = ref<Device | null>(null)
const defaultPosition = ref<number | null>(null)

function createDevice(position?: number): void {
  editingDevice.value = null
  defaultPosition.value = position ?? null
  deviceFormOpen.value = true
}

function editDevice(device: Device): void {
  editingDevice.value = device
  defaultPosition.value = null
  deviceFormOpen.value = true
}

async function removeDevice(device: Device): Promise<void> {
  if (!window.confirm(`确定删除设备「${device.name}」吗？`)) return
  try {
    await store.deleteDevice(device.id)
    store.notify('设备已删除')
  } catch (caught) {
    store.notify(caught instanceof Error ? caught.message : '删除失败')
  }
}

async function removeRack(): Promise<void> {
  if (!rack.value) return
  if (
    !window.confirm(
      `确定删除机柜「${rack.value.name}」吗？机柜内 ${devices.value.length} 台设备会保留，但会解除机柜与 U 位绑定。`,
    )
  ) {
    return
  }
  try {
    await store.deleteRack(rack.value.id)
    store.notify('机柜已删除')
    await router.push('/racks')
  } catch (caught) {
    store.notify(caught instanceof Error ? caught.message : '删除失败')
  }
}
</script>

<template>
  <div v-if="rack" class="space-y-5">
    <button
      class="flex items-center gap-1.5 text-xs text-slate-500 transition hover:text-slate-300"
      @click="router.push('/racks')"
    >
      <ArrowLeft class="h-3.5 w-3.5" />
      返回机柜列表
    </button>

    <div class="card flex flex-wrap items-start justify-between gap-4 p-5">
      <div>
        <div class="flex items-center gap-3">
          <h2 class="text-lg font-semibold tracking-tight text-slate-100">
            {{ rack.name }}
          </h2>
          <span class="badge bg-white/5 text-slate-400 ring-white/10">
            {{ rack.totalU }}U
          </span>
        </div>
        <p class="mt-1.5 flex items-center gap-1.5 text-[11px] text-slate-500">
          <MapPin class="h-3 w-3" />
          {{ rack.location ?? '未填写位置' }}
        </p>
        <p v-if="rack.note" class="mt-2 max-w-2xl text-xs leading-5 text-slate-400">
          {{ rack.note }}
        </p>
      </div>

      <div class="flex flex-wrap items-center gap-2">
        <button class="btn btn-ghost" @click="rackFormOpen = true">
          <Pencil class="h-3.5 w-3.5" />
          编辑机柜
        </button>
        <button class="btn btn-danger" @click="removeRack">
          <Trash2 class="h-3.5 w-3.5" />
          删除
        </button>
        <button class="btn btn-primary" @click="createDevice()">
          <Plus class="h-3.5 w-3.5" />
          放入设备
        </button>
      </div>
    </div>

    <div class="grid grid-cols-4 gap-4">
      <div class="card p-4">
        <p class="text-[10px] tracking-wide text-slate-500">已用 U 位</p>
        <p class="mt-1 text-xl font-semibold text-slate-100">
          {{ usedU }}<span class="ml-1 text-xs font-normal text-slate-500">U</span>
        </p>
      </div>
      <div class="card p-4">
        <p class="text-[10px] tracking-wide text-slate-500">剩余 U 位</p>
        <p class="mt-1 text-xl font-semibold text-slate-100">
          {{ Math.max(0, rack.totalU - usedU)
          }}<span class="ml-1 text-xs font-normal text-slate-500">U</span>
        </p>
      </div>
      <div class="card p-4">
        <p class="text-[10px] tracking-wide text-slate-500">设备数量</p>
        <p class="mt-1 text-xl font-semibold text-slate-100">
          {{ devices.length }}<span class="ml-1 text-xs font-normal text-slate-500">台</span>
        </p>
      </div>
      <div class="card p-4">
        <p class="text-[10px] tracking-wide text-slate-500">容量占用</p>
        <p
          class="mt-1 text-xl font-semibold"
          :class="pct > 85 ? 'text-rose-300' : pct > 60 ? 'text-amber-300' : 'text-emerald-300'"
        >
          {{ pct }}<span class="ml-0.5 text-xs font-normal text-slate-500">%</span>
        </p>
      </div>
    </div>

    <div class="grid grid-cols-3 gap-4">
      <section class="card col-span-2 p-5">
        <div class="mb-4 flex items-center justify-between">
          <div>
            <h3 class="text-xs font-semibold tracking-wide text-slate-200">U 位视图</h3>
            <p class="mt-0.5 text-[10px] text-slate-500">
              从顶部 U{{ rack.totalU }} 向下排列，点击空位可直接放置设备
            </p>
          </div>
          <span class="font-mono text-[10px] text-slate-600">
            FRONT VIEW
          </span>
        </div>

        <div class="mx-auto max-w-xl rounded-lg border border-white/5 bg-slate-950/60 p-3">
          <RackGrid
            :rack="rack"
            :devices="devices"
            @slot-click="createDevice"
            @device-click="editDevice"
          />
        </div>
      </section>

      <section class="card flex max-h-[36rem] flex-col p-5">
        <div class="flex items-center justify-between">
          <h3 class="text-xs font-semibold tracking-wide text-slate-200">设备清单</h3>
          <span class="text-[10px] text-slate-500">{{ devices.length }} 台</span>
        </div>

        <div v-if="devices.length" class="mt-3 space-y-2 overflow-y-auto pr-1">
          <div
            v-for="device in devices"
            :key="device.id"
            class="group rounded-lg border border-white/5 bg-white/[0.02] p-3 transition hover:border-white/10"
          >
            <div class="flex items-start justify-between gap-2">
              <div class="min-w-0">
                <p class="truncate text-xs font-medium text-slate-200">
                  {{ device.name }}
                </p>
                <p class="mt-0.5 truncate text-[10px] text-slate-500">
                  {{ deviceKindLabel(device.kind) }} ·
                  {{ deviceStatusLabel(device.status) }}
                </p>
              </div>
              <div class="flex shrink-0 gap-1 opacity-0 transition group-hover:opacity-100">
                <button
                  class="rounded p-1 text-slate-400 hover:bg-white/10 hover:text-slate-200"
                  title="编辑"
                  @click="editDevice(device)"
                >
                  <Pencil class="h-3 w-3" />
                </button>
                <button
                  class="rounded p-1 text-slate-500 hover:bg-rose-500/10 hover:text-rose-300"
                  title="删除"
                  @click="removeDevice(device)"
                >
                  <Trash2 class="h-3 w-3" />
                </button>
              </div>
            </div>
            <div class="mt-2 flex items-center justify-between text-[10px] text-slate-500">
              <span class="font-mono">{{ device.ips[0] ?? '—' }}</span>
              <span v-if="device.rackPosition" class="font-mono">
                U{{ device.rackPosition }}–U{{ device.rackPosition + device.rackHeight - 1 }}
              </span>
            </div>
          </div>
        </div>

        <div v-else class="flex flex-1 flex-col items-center justify-center gap-2 py-10 text-center">
          <p class="text-xs text-slate-500">机柜还是空的</p>
          <button class="btn btn-ghost mt-1" @click="createDevice()">
            <Plus class="h-3.5 w-3.5" />
            放入第一台设备
          </button>
        </div>
      </section>
    </div>

    <RackFormModal
      :open="rackFormOpen"
      :rack="rack"
      @close="rackFormOpen = false"
      @saved="rackFormOpen = false"
    />
    <DeviceFormModal
      :open="deviceFormOpen"
      :device="editingDevice"
      :default-rack-id="rack.id"
      :default-position="defaultPosition"
      @close="deviceFormOpen = false"
      @saved="deviceFormOpen = false"
    />
  </div>

  <div v-else class="card flex flex-col items-center gap-3 px-6 py-20 text-center">
    <p class="text-sm text-slate-300">机柜不存在或已被删除</p>
    <button class="btn btn-ghost" @click="router.push('/racks')">返回机柜列表</button>
  </div>
</template>
