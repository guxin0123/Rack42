import { computed, inject, provide, type ComputedRef } from 'vue'

const TOPOLOGY_HIGHLIGHT = Symbol('rack42-topology-highlight')

/**
 * Shares the endpoints of the currently selected link with custom node
 * components so the two connected devices can be highlighted.
 */
export function provideTopologyHighlight(deviceIds: ComputedRef<number[]>): void {
  provide(
    TOPOLOGY_HIGHLIGHT,
    computed(() => new Set(deviceIds.value)),
  )
}

export function useTopologyHighlight(): ComputedRef<Set<number>> {
  return inject(TOPOLOGY_HIGHLIGHT, computed(() => new Set<number>()))
}
