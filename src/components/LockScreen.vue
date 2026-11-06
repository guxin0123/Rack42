<script setup lang="ts">
import { KeyRound, Loader2, ShieldCheck } from 'lucide-vue-next'
import { computed, ref } from 'vue'

import { useAppStore } from '@/stores/app'

const store = useAppStore()

const password = ref('')
const confirm = ref('')
const sampleData = ref(false)
const busy = ref(false)
const error = ref('')

const isSetup = computed(() => !store.initialized)
const title = computed(() => (isSetup.value ? '创建主密码' : '解锁 Rack42'))
const subtitle = computed(() =>
  isSetup.value
    ? '主密码用于派生加密密钥，保护凭据中的密码字段。'
    : '输入主密码以解密本机凭据并进入控制台。',
)

async function submit(): Promise<void> {
  error.value = ''

  if (isSetup.value) {
    if (password.value.length < 4) {
      error.value = '主密码至少需要 4 个字符'
      return
    }
    if (password.value !== confirm.value) {
      error.value = '两次输入的主密码不一致'
      return
    }
  } else if (!password.value) {
    error.value = '请输入主密码'
    return
  }

  busy.value = true
  try {
    if (isSetup.value) {
      await store.setup(password.value, sampleData.value)
    } else {
      await store.unlock(password.value)
    }
    password.value = ''
    confirm.value = ''
  } catch (caught) {
    error.value = caught instanceof Error ? caught.message : '操作失败，请重试'
  } finally {
    busy.value = false
  }
}
</script>

<template>
  <div
    class="relative flex h-full items-center justify-center overflow-hidden bg-slate-950 px-6"
  >
    <div
      class="pointer-events-none absolute -left-40 -top-40 h-[32rem] w-[32rem] rounded-full bg-emerald-500/10 blur-3xl"
    />
    <div
      class="pointer-events-none absolute -bottom-40 -right-32 h-[28rem] w-[28rem] rounded-full bg-sky-500/10 blur-3xl"
    />

    <div class="relative w-full max-w-md">
      <div class="mb-8 flex items-center gap-3">
        <div
          class="flex h-12 w-12 items-center justify-center rounded-2xl border border-emerald-400/30 bg-emerald-400/10 text-lg font-bold tracking-tight text-emerald-300"
        >
          42
        </div>
        <div>
          <p class="text-lg font-semibold tracking-tight text-slate-100">Rack42</p>
          <p class="text-xs tracking-widest text-slate-500">LOCAL RACK CONTROL</p>
        </div>
      </div>

      <div
        class="rounded-2xl border border-white/10 bg-slate-900/70 p-6 shadow-2xl backdrop-blur"
      >
        <div class="mb-5 flex items-start gap-3">
          <div
            class="mt-0.5 flex h-9 w-9 items-center justify-center rounded-lg bg-emerald-500/10 text-emerald-300 ring-1 ring-emerald-500/20"
          >
            <ShieldCheck class="h-4 w-4" />
          </div>
          <div>
            <h1 class="text-base font-semibold text-slate-100">{{ title }}</h1>
            <p class="mt-1 text-xs leading-5 text-slate-400">{{ subtitle }}</p>
          </div>
        </div>

        <form class="space-y-4" @submit.prevent="submit">
          <label class="block">
            <span class="mb-1.5 block text-xs font-medium text-slate-400">主密码</span>
            <input
              v-model="password"
              type="password"
              autocomplete="current-password"
              class="w-full rounded-lg border border-white/10 bg-slate-950/70 px-3 py-2.5 text-sm text-slate-100 outline-none transition placeholder:text-slate-600 focus:border-emerald-400/60 focus:ring-2 focus:ring-emerald-400/10"
              placeholder="至少 4 个字符"
              :disabled="busy"
            />
          </label>

          <label v-if="isSetup" class="block">
            <span class="mb-1.5 block text-xs font-medium text-slate-400">确认主密码</span>
            <input
              v-model="confirm"
              type="password"
              autocomplete="new-password"
              class="w-full rounded-lg border border-white/10 bg-slate-950/70 px-3 py-2.5 text-sm text-slate-100 outline-none transition placeholder:text-slate-600 focus:border-emerald-400/60 focus:ring-2 focus:ring-emerald-400/10"
              placeholder="再次输入主密码"
              :disabled="busy"
            />
          </label>

          <label
            v-if="isSetup"
            class="flex items-start gap-3 rounded-lg border border-white/10 bg-slate-950/40 px-3 py-2.5"
          >
            <input
              v-model="sampleData"
              type="checkbox"
              class="mt-0.5 h-3.5 w-3.5 shrink-0 accent-emerald-500"
              :disabled="busy"
            />
            <span>
              <span class="block text-xs font-medium text-slate-300">生成示例数据</span>
              <span class="mt-0.5 block text-[10px] leading-4 text-slate-500">
                包含 3 个机柜、15 台设备与虚拟机、连接关系及示例凭据，方便快速体验；可随时删除。
              </span>
            </span>
          </label>

          <p
            v-if="error"
            class="rounded-lg border border-rose-500/20 bg-rose-500/10 px-3 py-2 text-xs leading-5 text-rose-300"
          >
            {{ error }}
          </p>

          <button
            type="submit"
            class="flex w-full items-center justify-center gap-2 rounded-lg bg-emerald-500 px-4 py-2.5 text-sm font-semibold text-emerald-950 transition hover:bg-emerald-400 disabled:cursor-not-allowed disabled:opacity-60"
            :disabled="busy"
          >
            <Loader2 v-if="busy" class="h-4 w-4 animate-spin" />
            <KeyRound v-else class="h-4 w-4" />
            {{ isSetup ? '创建并进入' : '解锁' }}
          </button>
        </form>

        <p class="mt-5 border-t border-white/5 pt-4 text-[11px] leading-5 text-slate-500">
          主密码不会离开本机，也不会被保存；一旦忘记将无法解密凭据，请务必牢记。
        </p>
      </div>

      <p class="mt-6 text-center text-[11px] tracking-wide text-slate-600">
        Rack42 · Tauri 2 · 数据仅存于本机
      </p>
    </div>
  </div>
</template>
