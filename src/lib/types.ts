export type DeviceKind =
  | 'physical_server'
  | 'vm'
  | 'router'
  | 'switch'
  | 'firewall'
  | 'storage'
  | 'pdu'
  | 'other'

export type DeviceStatus = 'active' | 'maintenance' | 'offline' | 'retired'

export type OsFamily = 'windows' | 'linux' | 'other'

export type LinkType = 'ethernet' | 'fiber' | 'power' | 'mgmt'

export interface BootstrapState {
  initialized: boolean
  unlocked: boolean
  dataDir: string
  portable: boolean
}

export interface Rack {
  id: number
  name: string
  location: string | null
  totalU: number
  note: string | null
}

export interface RackInput {
  name: string
  location?: string | null
  totalU?: number | null
  note?: string | null
}

export interface Device {
  id: number
  name: string
  kind: DeviceKind
  ips: string[]
  mac: string | null
  vendor: string | null
  model: string | null
  serial: string | null
  os: string | null
  osFamily: OsFamily | null
  status: DeviceStatus
  rackId: number | null
  rackPosition: number | null
  rackHeight: number
  hostId: number | null
  note: string | null
  createdAt: string
  updatedAt: string
}

export interface DeviceInput {
  name: string
  kind: DeviceKind
  ips?: string[]
  mac?: string | null
  vendor?: string | null
  model?: string | null
  serial?: string | null
  os?: string | null
  osFamily?: OsFamily | null
  status?: DeviceStatus | null
  rackId?: number | null
  rackPosition?: number | null
  rackHeight?: number | null
  hostId?: number | null
  note?: string | null
}

export type CredentialAction = 'rdp' | 'ssh' | 'sftp' | 'web'

export interface CredentialType {
  id: number
  slug: string
  name: string
  icon: string
  actions: CredentialAction[]
  defaultPort: number | null
  isBuiltin: boolean
  sortOrder: number
}

export interface CredentialTypeInput {
  name: string
  slug?: string | null
  icon?: string | null
  actions: CredentialAction[]
  defaultPort?: number | null
  sortOrder?: number | null
}

export interface Credential {
  id: number
  deviceId: number
  typeId: number
  label: string | null
  address: string | null
  port: number | null
  username: string | null
  domain: string | null
  hasPassword: boolean
  note: string | null
  isDefault: boolean
}

export interface CredentialInput {
  deviceId: number
  typeId?: number | null
  label?: string | null
  address?: string | null
  port?: number | null
  username?: string | null
  domain?: string | null
  password?: string | null
  note?: string | null
  isDefault?: boolean
}

export interface ConnectionTypeSummary {
  typeId: number
  slug: string
  name: string
  icon: string
  actions: CredentialAction[]
  credentialCount: number
  defaultCredentialId: number
}

export interface DeviceConnectionSummary {
  deviceId: number
  types: ConnectionTypeSummary[]
}

export interface ConnectResult {
  action: string
  target: string
}

export interface Link {
  id: number
  srcDeviceId: number
  srcPort: string | null
  dstDeviceId: number
  dstPort: string | null
  linkType: LinkType | null
  speed: string | null
  note: string | null
}

export interface LinkInput {
  srcDeviceId: number
  srcPort?: string | null
  dstDeviceId: number
  dstPort?: string | null
  linkType?: LinkType | null
  speed?: string | null
  note?: string | null
}

export interface Stats {
  rackCount: number
  deviceCount: number
  credentialCount: number
  linkCount: number
}

export interface LayoutNode {
  deviceId: number
  x: number
  y: number
}
