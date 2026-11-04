import type { OsFamily } from './types'

export interface OsFamilyGroup {
  value: OsFamily
  label: string
  options: string[]
}

export const osFamilyGroups: OsFamilyGroup[] = [
  {
    value: 'windows',
    label: 'Windows',
    options: [
      'Windows Server 2025',
      'Windows Server 2022',
      'Windows Server 2019',
      'Windows Server 2016',
      'Windows 11',
      'Windows 10',
    ],
  },
  {
    value: 'linux',
    label: 'Linux',
    options: [
      'Ubuntu 24.04 LTS',
      'Ubuntu 22.04 LTS',
      'Debian 12',
      'Rocky Linux 9',
      'AlmaLinux 9',
      'RHEL 9',
      'CentOS Stream 9',
      'openSUSE Leap 15',
      'Arch Linux',
    ],
  },
  {
    value: 'other',
    label: '其他',
    options: ['VMware ESXi', 'Proxmox VE', 'FreeBSD', 'macOS', '网络设备系统', '其他'],
  },
]

const linuxPattern =
  /(linux|ubuntu|debian|centos|rhel|red hat|rocky|alma|suse|arch|fedora|kylin|uos|openeuler)/i

export function osFamilyOf(os: string | null | undefined): OsFamily {
  if (!os) return 'other'
  if (/windows/i.test(os)) return 'windows'
  if (linuxPattern.test(os)) return 'linux'
  return 'other'
}

export function osOptionsFor(family: OsFamily): string[] {
  return osFamilyGroups.find((group) => group.value === family)?.options ?? []
}
