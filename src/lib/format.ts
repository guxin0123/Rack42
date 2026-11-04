import type { DeviceKind, DeviceStatus, LinkType } from './types'

interface KindMeta {
  label: string
  color: string
  badge: string
}

export const deviceKindMeta: Record<DeviceKind, KindMeta> = {
  physical_server: {
    label: '物理服务器',
    color: '#38bdf8',
    badge: 'bg-sky-500/15 text-sky-300 ring-sky-500/30',
  },
  vm: {
    label: '虚拟机',
    color: '#a78bfa',
    badge: 'bg-violet-500/15 text-violet-300 ring-violet-500/30',
  },
  router: {
    label: '路由器',
    color: '#f59e0b',
    badge: 'bg-amber-500/15 text-amber-300 ring-amber-500/30',
  },
  switch: {
    label: '交换机',
    color: '#34d399',
    badge: 'bg-emerald-500/15 text-emerald-300 ring-emerald-500/30',
  },
  firewall: {
    label: '防火墙',
    color: '#fb7185',
    badge: 'bg-rose-500/15 text-rose-300 ring-rose-500/30',
  },
  storage: {
    label: '存储',
    color: '#22d3ee',
    badge: 'bg-cyan-500/15 text-cyan-300 ring-cyan-500/30',
  },
  pdu: {
    label: 'PDU',
    color: '#facc15',
    badge: 'bg-yellow-500/15 text-yellow-300 ring-yellow-500/30',
  },
  other: {
    label: '其他',
    color: '#94a3b8',
    badge: 'bg-slate-500/15 text-slate-300 ring-slate-500/30',
  },
}

export const deviceStatusMeta: Record<DeviceStatus, { label: string; dot: string }> = {
  active: { label: '运行中', dot: 'bg-emerald-400' },
  maintenance: { label: '维护中', dot: 'bg-amber-400' },
  offline: { label: '已离线', dot: 'bg-slate-500' },
  retired: { label: '已退役', dot: 'bg-rose-400' },
}

export const linkTypeMeta: Record<LinkType, { label: string; color: string }> = {
  ethernet: { label: '以太网', color: '#34d399' },
  fiber: { label: '光纤', color: '#22d3ee' },
  power: { label: '电源', color: '#f59e0b' },
  mgmt: { label: '管理', color: '#a78bfa' },
}

export function deviceKindLabel(kind: string): string {
  return deviceKindMeta[kind as DeviceKind]?.label ?? kind
}

export function deviceKindColor(kind: string): string {
  return deviceKindMeta[kind as DeviceKind]?.color ?? '#94a3b8'
}

export function deviceStatusLabel(status: string): string {
  return deviceStatusMeta[status as DeviceStatus]?.label ?? status
}

export function linkTypeLabel(type: string | null): string {
  if (!type) return '未分类'
  return linkTypeMeta[type as LinkType]?.label ?? type
}

export function linkTypeColor(type: string | null): string {
  if (!type) return '#64748b'
  return linkTypeMeta[type as LinkType]?.color ?? '#64748b'
}

export function formatDate(value: string | null): string {
  if (!value) return '—'
  const normalized = value.includes('T') ? value : value.replace(' ', 'T')
  const date = new Date(
    normalized.endsWith('Z') || /[+-]\d{2}:\d{2}$/.test(normalized)
      ? normalized
      : `${normalized}Z`,
  )
  if (Number.isNaN(date.getTime())) return value
  return new Intl.DateTimeFormat('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  }).format(date)
}
