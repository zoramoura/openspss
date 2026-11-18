import * as Comlink from 'comlink'
import init, { compute_descriptives } from '../wasm/engine'

const statsApi = {
  async initEngine() {
    await init()
  },
  async getDescriptives(numbers: number[]) {
    const floatArray = new Float64Array(numbers)
    return compute_descriptives(floatArray)
  },
}

export type StatsWorkerApi = typeof statsApi
Comlink.expose(statsApi)
