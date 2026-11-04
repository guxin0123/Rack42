import { invoke } from '@tauri-apps/api/core'

import type {
  BootstrapState,
  ConnectResult,
  Credential,
  CredentialAction,
  CredentialInput,
  CredentialType,
  CredentialTypeInput,
  Device,
  DeviceConnectionSummary,
  DeviceInput,
  LayoutNode,
  Link,
  LinkInput,
  Rack,
  RackInput,
  Stats,
} from './types'

async function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(command, args)
  } catch (error) {
    const message =
      typeof error === 'string'
        ? error
        : error instanceof Error
          ? error.message
          : '操作失败，请稍后重试'
    throw new Error(message)
  }
}

export const api = {
  bootstrap: () => call<BootstrapState>('get_bootstrap_state'),
  setupMasterPassword: (password: string) =>
    call<void>('setup_master_password', { password }),
  unlock: (password: string) => call<void>('unlock', { password }),
  lock: () => call<void>('lock'),

  listRacks: () => call<Rack[]>('list_racks'),
  createRack: (input: RackInput) => call<Rack>('create_rack', { input }),
  updateRack: (id: number, input: RackInput) => call<Rack>('update_rack', { id, input }),
  deleteRack: (id: number) => call<void>('delete_rack', { id }),

  listDevices: () => call<Device[]>('list_devices'),
  listDeviceConnections: () =>
    call<DeviceConnectionSummary[]>('list_device_connections'),
  connectCredential: (credentialId: number, action: CredentialAction) =>
    call<ConnectResult>('connect_credential', { credentialId, action }),
  createDevice: (input: DeviceInput) => call<Device>('create_device', { input }),
  updateDevice: (id: number, input: DeviceInput) =>
    call<Device>('update_device', { id, input }),
  deleteDevice: (id: number) => call<void>('delete_device', { id }),

  listLinks: () => call<Link[]>('list_links'),
  createLink: (input: LinkInput) => call<Link>('create_link', { input }),
  deleteLink: (id: number) => call<void>('delete_link', { id }),

  listCredentialTypes: () => call<CredentialType[]>('list_credential_types'),
  createCredentialType: (input: CredentialTypeInput) =>
    call<CredentialType>('create_credential_type', { input }),
  updateCredentialType: (id: number, input: CredentialTypeInput) =>
    call<CredentialType>('update_credential_type', { id, input }),
  deleteCredentialType: (id: number) => call<void>('delete_credential_type', { id }),

  listCredentials: (deviceId: number) =>
    call<Credential[]>('list_credentials', { deviceId }),
  createCredential: (input: CredentialInput) =>
    call<Credential>('create_credential', { input }),
  updateCredential: (id: number, input: CredentialInput) =>
    call<Credential>('update_credential', { id, input }),
  deleteCredential: (id: number) => call<void>('delete_credential', { id }),
  revealCredential: (id: number) => call<string>('reveal_credential', { id }),

  stats: () => call<Stats>('get_stats'),
  exportData: () => call<string>('export_data'),

  loadLayout: (view: string) => call<LayoutNode[]>('load_layout', { view }),
  saveLayout: (view: string, nodes: LayoutNode[]) =>
    call<void>('save_layout', { view, nodes }),
}
