import { defineStore } from 'pinia'
import { computed, ref } from 'vue'

export type ThemeMode = 'dark' | 'light' | 'system'
export type ResolvedTheme = 'dark' | 'light'

const STORAGE_KEY = 'rack42.theme'

function readMode(): ThemeMode {
  try {
    const stored = localStorage.getItem(STORAGE_KEY)
    if (stored === 'dark' || stored === 'light' || stored === 'system') {
      return stored
    }
  } catch {
    // ignore storage failures
  }
  return 'system'
}

export const useThemeStore = defineStore('theme', () => {
  const media =
    typeof window !== 'undefined'
      ? window.matchMedia('(prefers-color-scheme: dark)')
      : undefined

  const mode = ref<ThemeMode>(readMode())
  const systemDark = ref(media?.matches ?? true)

  const resolved = computed<ResolvedTheme>(() => {
    if (mode.value === 'system') {
      return systemDark.value ? 'dark' : 'light'
    }
    return mode.value
  })

  function apply(): void {
    document.documentElement.dataset.theme = resolved.value
    document.documentElement.style.colorScheme = resolved.value
  }

  function setMode(next: ThemeMode): void {
    mode.value = next
    try {
      localStorage.setItem(STORAGE_KEY, next)
    } catch {
      // ignore storage failures
    }
    apply()
  }

  media?.addEventListener('change', (event) => {
    systemDark.value = event.matches
    if (mode.value === 'system') apply()
  })

  apply()

  return { mode, resolved, setMode }
})
