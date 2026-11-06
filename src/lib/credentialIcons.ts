import {
  Cloud,
  Database,
  Globe,
  HardDrive,
  KeyRound,
  Monitor,
  Server,
  Shield,
  Terminal,
} from 'lucide-vue-next'
import type { Component } from 'vue'

export const credentialIconOptions: Array<{
  value: string
  label: string
  icon: Component
}> = [
  { value: 'monitor', label: '显示器', icon: Monitor },
  { value: 'terminal', label: '终端', icon: Terminal },
  { value: 'server', label: '服务器', icon: Server },
  { value: 'cloud', label: '云', icon: Cloud },
  { value: 'globe', label: 'Web', icon: Globe },
  { value: 'key', label: '密钥', icon: KeyRound },
  { value: 'database', label: '数据库', icon: Database },
  { value: 'hard-drive', label: '存储', icon: HardDrive },
  { value: 'shield', label: '安全', icon: Shield },
]

export function credentialIconComponent(icon: string | null | undefined): Component {
  return (
    credentialIconOptions.find((option) => option.value === icon)?.icon ?? KeyRound
  )
}

export function isCustomCredentialIcon(icon: string | null | undefined): boolean {
  return typeof icon === 'string' && icon.startsWith('data:image/')
}
