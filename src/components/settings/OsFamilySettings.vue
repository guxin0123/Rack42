<script setup lang="ts">
import { Pencil, Plus, Trash2 } from 'lucide-vue-next'
import { reactive, ref } from 'vue'

import Modal from '@/components/ui/Modal.vue'
import { iconComponent, iconOptions, isCustomIcon } from '@/lib/icons'
import { api } from '@/lib/ipc'
import type { OsFamilyInfo, OsFamilyInput } from '@/lib/types'
import { useAppStore } from '@/stores/app'

const store = useAppStore()

const modalOpen = ref(false)
const editing = ref<OsFamilyInfo | null>(null)
const saving = ref(false)
const error = ref('')

const form = reactive({
  name: '',
  icon: 'monitor',
  optionsText: '',
  sortOrder: '0',
})

function openCreate(): void {
  editing.value = null
  form.name = ''
  form.icon = 'monitor'
  form.optionsText = ''
  form.sortOrder = String((store.osFamilies.at(-1)?.sortOrder ?? 0) + 10)
  error.value = ''
  modalOpen.value = true
}

function openEdit(family: OsFamilyInfo): void {
  editing.value = family
  form.name = family.name
  form.icon = family.icon
  form.optionsText = family.options.join('\n')
  form.sortOrder = String(family.sortOrder)
  error.value = ''
  modalOpen.value = true
}

function parseOptions(value: string): string[] {
  return value
    .split('\n')
    .map((line) => line.trim())
    .filter(Boolean)
}

async function submit(): Promise<void> {
  if (!form.name.trim()) {
    error.value = '请输入分类名称'
    return
  }

  const input: OsFamilyInput = {
    name: form.name.trim(),
    icon: form.icon,
    options: parseOptions(form.optionsText),
    sortOrder: Number(form.sortOrder) || 0,
  }

  saving.value = true
  error.value = ''
  try {
    if (editing.value) {
      await api.updateOsFamily(editing.value.id, input)
      store.notify('操作系统分类已更新')
    } else {
      await api.createOsFamily(input)
      store.notify('操作系统分类已创建')
    }
    await store.refresh()
    modalOpen.value = false
  } catch (caught) {
    error.value = caught instanceof Error ? caught.message : '保存失败'
  } finally {
    saving.value = false
  }
}

async function remove(family: OsFamilyInfo): Promise<void> {
  if (
    !window.confirm(
      `确定删除操作系统分类「${family.name}」吗？使用该分类的设备会归入「其他」。`,
    )
  ) {
    return
  }

  try {
    await api.deleteOsFamily(family.id)
    await store.refresh()
    store.notify('操作系统分类已删除')
  } catch (caught) {
    store.notify(caught instanceof Error ? caught.message : '删除失败')
  }
}
</script>

<template>
  <section class="card p-5">
    <div class="flex flex-wrap items-center justify-between gap-3">
      <div>
        <h3 class="text-xs font-semibold tracking-wide text-slate-200">操作系统分类</h3>
        <p class="mt-0.5 text-[10px] text-slate-500">
          分类名称、图标与具体系统列表；图标会显示在设备列表和拓扑中。内置分类不可删除。
        </p>
      </div>
      <button class="btn btn-ghost" type="button" @click="openCreate">
        <Plus class="h-3.5 w-3.5" />
        新增分类
      </button>
    </div>

    <div class="mt-4 overflow-x-auto">
      <table v-if="store.osFamilies.length" class="w-full text-left text-xs">
        <thead class="text-[10px] uppercase tracking-wider text-slate-500">
          <tr class="border-b border-white/5">
            <th class="py-2.5 pr-4 font-medium">分类</th>
            <th class="px-2 py-2.5 font-medium">具体系统</th>
            <th class="px-2 py-2.5 font-medium">排序</th>
            <th class="py-2.5 pl-4 text-right font-medium">操作</th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="family in store.osFamilies"
            :key="family.id"
            class="border-b border-white/[0.03] last:border-0"
          >
            <td class="py-2.5 pr-4">
              <div class="flex items-center gap-2">
                <img
                  v-if="isCustomIcon(family.icon)"
                  :src="family.icon"
                  alt=""
                  class="h-4 w-4 rounded object-cover"
                />
                <component
                  :is="iconComponent(family.icon)"
                  v-else
                  class="h-3.5 w-3.5 text-slate-400"
                />
                <span class="font-medium text-slate-200">{{ family.name }}</span>
                <span
                  v-if="family.isBuiltin"
                  class="rounded bg-white/5 px-1.5 py-0.5 text-[9px] text-slate-500"
                >
                  内置
                </span>
              </div>
            </td>
            <td class="px-2 py-2.5 text-slate-500">
              <span v-if="family.options.length">
                {{ family.options.length }} 个 · {{ family.options.slice(0, 3).join(' / ')
                }}{{ family.options.length > 3 ? ' …' : '' }}
              </span>
              <span v-else class="text-slate-600">未配置</span>
            </td>
            <td class="px-2 py-2.5 font-mono text-[10px] text-slate-500">
              {{ family.sortOrder }}
            </td>
            <td class="py-2.5 pl-4">
              <div class="flex justify-end gap-1">
                <button
                  class="rounded-md p-1.5 text-slate-400 transition hover:bg-white/5 hover:text-slate-200"
                  title="编辑"
                  type="button"
                  @click="openEdit(family)"
                >
                  <Pencil class="h-3.5 w-3.5" />
                </button>
                <button
                  class="rounded-md p-1.5 text-slate-500 transition hover:bg-rose-500/10 hover:text-rose-300 disabled:cursor-not-allowed disabled:opacity-30"
                  :disabled="family.isBuiltin"
                  :title="family.isBuiltin ? '内置分类不可删除' : '删除'"
                  type="button"
                  @click="remove(family)"
                >
                  <Trash2 class="h-3.5 w-3.5" />
                </button>
              </div>
            </td>
          </tr>
        </tbody>
      </table>
      <p v-else class="py-8 text-center text-xs text-slate-500">暂无操作系统分类</p>
    </div>

    <Modal
      :open="modalOpen"
      :title="editing ? '编辑操作系统分类' : '新增操作系统分类'"
      width="max-w-xl"
      @close="modalOpen = false"
    >
      <form class="space-y-4" @submit.prevent="submit">
        <div class="grid grid-cols-2 gap-4">
          <div>
            <label class="label" for="os-family-name">分类名称</label>
            <input
              id="os-family-name"
              v-model="form.name"
              class="input"
              placeholder="如 Windows / Linux / 网络设备"
              maxlength="40"
            />
          </div>
          <div>
            <label class="label" for="os-family-order">排序</label>
            <input
              id="os-family-order"
              v-model="form.sortOrder"
              class="input"
              type="number"
              step="10"
            />
          </div>
        </div>

        <div>
          <p class="label">图标</p>
          <div class="flex flex-wrap gap-2">
            <button
              v-for="option in iconOptions"
              :key="option.value"
              type="button"
              class="flex h-9 w-9 items-center justify-center rounded-lg border transition"
              :class="
                form.icon === option.value
                  ? 'border-emerald-400/60 bg-emerald-400/10 text-emerald-300'
                  : 'border-white/10 bg-white/[0.02] text-slate-400 hover:border-white/20 hover:text-slate-200'
              "
              :title="option.label"
              @click="form.icon = option.value"
            >
              <component :is="option.icon" class="h-4 w-4" />
            </button>
          </div>
        </div>

        <div>
          <label class="label" for="os-family-options">具体系统（每行一个）</label>
          <textarea
            id="os-family-options"
            v-model="form.optionsText"
            class="input resize-none font-mono text-[11px]"
            rows="7"
            placeholder="Windows Server 2022&#10;Windows 11"
          />
          <p class="mt-1.5 text-[10px] text-slate-500">
            这些条目会出现在设备表单的「具体系统」下拉框中，留空表示该分类不提供预置系统。
          </p>
        </div>

        <p
          v-if="error"
          class="rounded-lg border border-rose-500/20 bg-rose-500/10 px-3 py-2 text-xs text-rose-300"
        >
          {{ error }}
        </p>
      </form>

      <template #footer>
        <button class="btn btn-ghost" type="button" @click="modalOpen = false">取消</button>
        <button class="btn btn-primary" type="button" :disabled="saving" @click="submit">
          {{ saving ? '保存中…' : '保存' }}
        </button>
      </template>
    </Modal>
  </section>
</template>
