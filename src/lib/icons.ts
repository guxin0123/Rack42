import {
  Apple,
  Boxes,
  Cloud,
  Cpu,
  Database,
  Globe,
  HardDrive,
  KeyRound,
  Laptop,
  Monitor,
  Router,
  Server,
  Shield,
  Terminal,
} from 'lucide-vue-next'
import type { Component } from 'vue'

export const iconOptions: Array<{ value: string; label: string; icon: Component }> = [
  { value: 'monitor', label: '显示器', icon: Monitor },
  { value: 'terminal', label: '终端', icon: Terminal },
  { value: 'server', label: '服务器', icon: Server },
  { value: 'cloud', label: '云', icon: Cloud },
  { value: 'globe', label: 'Web', icon: Globe },
  { value: 'key', label: '密钥', icon: KeyRound },
  { value: 'database', label: '数据库', icon: Database },
  { value: 'hard-drive', label: '存储', icon: HardDrive },
  { value: 'shield', label: '安全', icon: Shield },
  { value: 'cpu', label: 'CPU', icon: Cpu },
  { value: 'boxes', label: '容器', icon: Boxes },
  { value: 'apple', label: 'Apple', icon: Apple },
  { value: 'laptop', label: '笔记本', icon: Laptop },
  { value: 'router', label: '路由', icon: Router },
]

export function iconComponent(icon: string | null | undefined): Component {
  return iconOptions.find((option) => option.value === icon)?.icon ?? KeyRound
}

export function isCustomIcon(icon: string | null | undefined): boolean {
  return typeof icon === 'string' && icon.startsWith('data:image/')
}
