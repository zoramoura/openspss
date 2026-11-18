import * as Comlink from 'comlink'
import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { StatsWorkerApi } from '../workers/stats.worker'

export const useStatsStore = defineStore('stats', () => {
  const ready = ref(false)
  const worker = new Worker(new URL('../workers/stats.worker.ts', import.meta.url), {
    type: 'module',
  })
  const statsApi = Comlink.wrap<Comlink.Remote<StatsWorkerApi>>(worker)
  let initialization: Promise<void> | undefined

  async function initEngine() {
    if (ready.value) return

    initialization ??= statsApi.initEngine()
      .then(() => {
        ready.value = true
      })
      .catch((error: unknown) => {
        initialization = undefined
        throw error
      })

    await initialization
  }

  async function getDescriptives(numbers: number[]) {
    await initEngine()
    return statsApi.getDescriptives(numbers)
  }

  return { ready, initEngine, getDescriptives }
})
