import { defineStore } from 'pinia'
import { computed, ref } from 'vue'

import { api } from '@/lib/ipc'
import type {
  CredentialType,
  Device,
  DeviceConnectionSummary,
  DeviceInput,
  Link,
  LinkInput,
  OsFamilyInfo,
  Rack,
  RackInput,
  Stats,
} from '@/lib/types'

function errorMessage(error: unknown, fallback: string): string {
  return error instanceof Error ? error.message : fallback
}

export const useAppStore = defineStore('app', () => {
  const loading = ref(true)
  const bootError = ref('')
  const initialized = ref(false)
  const unlocked = ref(false)
  const dataDir = ref('')
  const portable = ref(false)

  const racks = ref<Rack[]>([])
  const devices = ref<Device[]>([])
  const links = ref<Link[]>([])
  const credentialTypes = ref<CredentialType[]>([])
  const osFamilies = ref<OsFamilyInfo[]>([])
  const deviceConnections = ref<DeviceConnectionSummary[]>([])
  const stats = ref<Stats>({
    rackCount: 0,
    deviceCount: 0,
    credentialCount: 0,
    linkCount: 0,
  })

  const notice = ref('')
  let noticeTimer: ReturnType<typeof setTimeout> | undefined

  const rackById = computed(() => new Map(racks.value.map((rack) => [rack.id, rack])))
  const deviceById = computed(
    () => new Map(devices.value.map((device) => [device.id, device])),
  )

  function notify(message: string): void {
    notice.value = message
    if (noticeTimer) clearTimeout(noticeTimer)
    noticeTimer = setTimeout(() => {
      notice.value = ''
    }, 3200)
  }

  async function refresh(): Promise<void> {
    const [
      nextRacks,
      nextDevices,
      nextLinks,
      nextTypes,
      nextOsFamilies,
      nextConnections,
      nextStats,
    ] = await Promise.all([
      api.listRacks(),
      api.listDevices(),
      api.listLinks(),
      api.listCredentialTypes(),
      api.listOsFamilies(),
      api.listDeviceConnections(),
      api.stats(),
    ])
    racks.value = nextRacks
    devices.value = nextDevices
    links.value = nextLinks
    credentialTypes.value = nextTypes
    osFamilies.value = nextOsFamilies
    deviceConnections.value = nextConnections
    stats.value = nextStats
  }

  async function bootstrap(): Promise<void> {
    loading.value = true
    bootError.value = ''
    try {
      const state = await api.bootstrap()
      initialized.value = state.initialized
      unlocked.value = state.unlocked
      dataDir.value = state.dataDir
      portable.value = state.portable
      if (state.unlocked) await refresh()
    } catch (error) {
      bootError.value = `${errorMessage(
        error,
        '无法连接本地服务',
      )}。请通过 “pnpm tauri dev” 启动桌面应用。`
    } finally {
      loading.value = false
    }
  }

  async function setup(password: string): Promise<void> {
    await api.setupMasterPassword(password)
    initialized.value = true
    unlocked.value = true
    await refresh()
  }

  async function unlock(password: string): Promise<void> {
    await api.unlock(password)
    unlocked.value = true
    await refresh()
  }

  async function lock(): Promise<void> {
    await api.lock()
    unlocked.value = false
    racks.value = []
    devices.value = []
    links.value = []
    stats.value = {
      rackCount: 0,
      deviceCount: 0,
      credentialCount: 0,
      linkCount: 0,
    }
  }

  async function createRack(input: RackInput): Promise<Rack> {
    const rack = await api.createRack(input)
    await refresh()
    return rack
  }

  async function updateRack(id: number, input: RackInput): Promise<Rack> {
    const rack = await api.updateRack(id, input)
    await refresh()
    return rack
  }

  async function deleteRack(id: number): Promise<void> {
    await api.deleteRack(id)
    await refresh()
  }

  async function createDevice(input: DeviceInput): Promise<Device> {
    const device = await api.createDevice(input)
    await refresh()
    return device
  }

  async function updateDevice(id: number, input: DeviceInput): Promise<Device> {
    const device = await api.updateDevice(id, input)
    await refresh()
    return device
  }

  async function deleteDevice(id: number): Promise<void> {
    await api.deleteDevice(id)
    await refresh()
  }

  async function createLink(input: LinkInput): Promise<Link> {
    const link = await api.createLink(input)
    await refresh()
    return link
  }

  async function deleteLink(id: number): Promise<void> {
    await api.deleteLink(id)
    await refresh()
  }

  return {
    loading,
    bootError,
    initialized,
    unlocked,
    dataDir,
    portable,
    racks,
    devices,
    links,
    credentialTypes,
    osFamilies,
    deviceConnections,
    stats,
    notice,
    rackById,
    deviceById,
    notify,
    refresh,
    bootstrap,
    setup,
    unlock,
    lock,
    createRack,
    updateRack,
    deleteRack,
    createDevice,
    updateDevice,
    deleteDevice,
    createLink,
    deleteLink,
  }
})
