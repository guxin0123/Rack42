import type { OsFamilyInfo } from './types'

const windowsPattern = /windows/i
const linuxPattern =
  /(linux|ubuntu|debian|centos|rhel|red hat|rocky|alma|suse|arch|fedora|kylin|uos|openeuler)/i

export function detectOsFamily(
  os: string | null | undefined,
  families: OsFamilyInfo[],
): string {
  const fallback =
    families.find((family) => family.slug === 'other')?.slug ??
    families[0]?.slug ??
    'other'

  if (!os) return fallback

  if (
    windowsPattern.test(os) &&
    families.some((family) => family.slug === 'windows')
  ) {
    return 'windows'
  }

  if (linuxPattern.test(os) && families.some((family) => family.slug === 'linux')) {
    return 'linux'
  }

  return fallback
}

export function osOptionsFor(family: string, families: OsFamilyInfo[]): string[] {
  return families.find((item) => item.slug === family)?.options ?? []
}

export function osFamilyInfo(
  family: string | null | undefined,
  families: OsFamilyInfo[],
): OsFamilyInfo | undefined {
  return families.find((item) => item.slug === family)
}
