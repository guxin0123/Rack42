import { ref } from 'vue'

import { api } from '@/lib/ipc'
import type {
  ConnectionCredentialSummary,
  ConnectionTypeSummary,
  CredentialAction,
  Device,
} from '@/lib/types'
import { useAppStore } from '@/stores/app'

export const actionLabels: Record<string, string> = {
  rdp: '远程桌面',
  web: '打开 Web',
  ssh: 'SSH 终端',
  sftp: 'SFTP 文件传输',
}

export const actionStarted: Record<string, string> = {
  rdp: '已启动远程桌面',
  web: '已在浏览器中打开',
  ssh: '已启动 SSH 终端',
  sftp: '已启动 SFTP 客户端',
}

export interface CredentialChoice {
  deviceId: number
  type: ConnectionTypeSummary
  action: CredentialAction
}

/**
 * Shared entry point for every connection command (device list, topology
 * context menu). All actions end up in `connect_credential`, so auditing and
 * password handling stay identical everywhere.
 */
export function useDeviceConnect() {
  const store = useAppStore()

  const connectingKey = ref('')
  const chooserOpen = ref(false)
  const chooser = ref<CredentialChoice | null>(null)

  function connectionKey(deviceId: number, typeId: number, action: CredentialAction): string {
    return `${deviceId}:${typeId}:${action}`
  }

  function isConnecting(deviceId: number, typeId: number, action: CredentialAction): boolean {
    return connectingKey.value === connectionKey(deviceId, typeId, action)
  }

  function connectionTypes(deviceId: number): ConnectionTypeSummary[] {
    const types =
      store.deviceConnections.find((item) => item.deviceId === deviceId)?.types ?? []
    return types.filter((type) => type.actions.length > 0)
  }

  function preferredCredential(
    type: ConnectionTypeSummary,
  ): ConnectionCredentialSummary | undefined {
    return type.credentials.find((credential) => credential.isDefault) ?? type.credentials[0]
  }

  function closeChooser(): void {
    chooserOpen.value = false
    chooser.value = null
  }

  async function connect(
    deviceId: number,
    type: ConnectionTypeSummary,
    action: CredentialAction,
  ): Promise<void> {
    if (connectingKey.value) return

    if (type.credentials.length > 1) {
      chooser.value = { deviceId, type, action }
      chooserOpen.value = true
      return
    }

    const credential = preferredCredential(type)
    if (!credential) {
      store.notify('该连接类型还没有可用凭据')
      return
    }

    await startConnection(credential.id, action, deviceId, type.typeId)
  }

  /**
   * Double-click shortcut: first connection-capable credential type (ordered by
   * the backend's type sort order), its default/first credential, first action.
   */
  async function connectFirst(device: Device): Promise<void> {
    if (connectingKey.value) return

    const type = connectionTypes(device.id)[0]
    if (!type) {
      store.notify(`「${device.name}」还没有可用于连接的凭据`)
      return
    }

    const credential = preferredCredential(type)
    if (!credential) {
      store.notify(`「${device.name}」还没有可用于连接的凭据`)
      return
    }

    await startConnection(
      credential.id,
      type.actions[0],
      device.id,
      type.typeId,
    )
  }

  async function chooseCredential(credential: ConnectionCredentialSummary): Promise<void> {
    const context = chooser.value
    closeChooser()
    if (!context) return
    await startConnection(
      credential.id,
      context.action,
      context.deviceId,
      context.type.typeId,
    )
  }

  async function startConnection(
    credentialId: number,
    action: CredentialAction,
    deviceId: number,
    typeId: number,
  ): Promise<void> {
    if (connectingKey.value) return
    const key = connectionKey(deviceId, typeId, action)
    connectingKey.value = key

    try {
      const result = await api.connectCredential(credentialId, action)
      if (result.clipboard) {
        store.notify(
          `${actionStarted[action] ?? '已启动连接'}；密码已复制到剪贴板，10 秒后自动清除`,
        )
      } else {
        store.notify(actionStarted[action] ?? '已启动连接')
      }
    } catch (caught) {
      store.notify(caught instanceof Error ? caught.message : '连接失败')
    } finally {
      connectingKey.value = ''
    }
  }

  return {
    connectingKey,
    chooserOpen,
    chooser,
    connectionKey,
    isConnecting,
    connectionTypes,
    connect,
    connectFirst,
    chooseCredential,
    closeChooser,
  }
}
