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

export type OsFamily = string

export interface OsFamilyInfo {
  id: number
  slug: string
  name: string
  icon: string
  options: string[]
  isBuiltin: boolean
  sortOrder: number
}

export interface OsFamilyInput {
  name: string
  slug?: string | null
  icon?: string | null
  options?: string[]
  sortOrder?: number | null
}

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

export interface ConnectionCredentialSummary {
  id: number
  label: string | null
  username: string | null
  isDefault: boolean
}

export interface ConnectionTypeSummary {
  typeId: number
  slug: string
  name: string
  icon: string
  actions: CredentialAction[]
  credentialCount: number
  defaultCredentialId: number
  credentials: ConnectionCredentialSummary[]
}

export interface DeviceConnectionSummary {
  deviceId: number
  types: ConnectionTypeSummary[]
}

export interface ConnectResult {
  action: string
  target: string
}

export type SshClient = 'xshell' | 'putty' | 'openssh'

export type SftpClient = 'xftp' | 'psftp' | 'openssh'

export interface ConnectionSettings {
  sshClient: SshClient
  sshPath: string | null
  sftpClient: SftpClient
  sftpPath: string | null
  autoFillPassword: boolean
}

export interface AuditLog {
  id: number
  action: string
  deviceName: string | null
  credentialLabel: string | null
  credentialType: string | null
  target: string | null
  result: 'success' | 'failed'
  detail: string | null
  createdAt: string
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

export interface RackLayoutNode {
  rackId: number
  x: number
  y: number
}

export interface IpOverviewEntry {
  ip: string
  subnet: string
  deviceId: number
  deviceName: string
  source: 'device' | 'credential'
  credentialId: number | null
  credentialLabel: string | null
}

export interface PortProbe {
  credentialId: number
  label: string | null
  address: string
  port: number
  reachable: boolean
  latencyMs: number | null
  error: string | null
}

export interface DevicePortProbe {
  deviceId: number
  probes: PortProbe[]
  minLatencyMs: number | null
}
