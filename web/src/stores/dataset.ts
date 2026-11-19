import * as Comlink from 'comlink'
import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { DataWorkerApi, Dataset, Variable } from '../workers/data.worker'

export const useDatasetStore = defineStore('dataset', () => {
  const ready = ref(false)
  const loading = ref(false)
  const error = ref<string | null>(null)
  const dataset = ref<Dataset | null>(null)

  const worker = new Worker(
    new URL('../workers/data.worker.ts', import.meta.url),
    { type: 'module' },
  )
  const dataApi = Comlink.wrap<Comlink.Remote<DataWorkerApi>>(worker)
  let initialization: Promise<void> | undefined

  async function initEngine() {
    if (ready.value) return
    error.value = null

    initialization ??= dataApi
      .initEngine()
      .then(() => {
        ready.value = true
      })
      .catch((err: unknown) => {
        initialization = undefined
        const message =
          err instanceof Error ? err.message : String(err)
        error.value = `Failed to initialise data engine: ${message}`
        throw err
      })

    await initialization
  }

  async function loadFile(buffer: ArrayBuffer, fileName: string) {
    await initEngine()
    loading.value = true
    error.value = null
    try {
      const ds = await dataApi.readFile(buffer, fileName)
      dataset.value = ds
      return ds
    } catch (err: unknown) {
      const message = err instanceof Error ? err.message : String(err)
      error.value = `Failed to load "${fileName}": ${message}`
      throw err
    } finally {
      loading.value = false
    }
  }

  /** Patch metadata on a single variable by index. */
  function setVariableMetadata(index: number, patch: Partial<Variable>) {
    const ds = dataset.value
    if (!ds) throw new Error('No dataset loaded')
    const variable = ds.variables[index]
    if (!variable) throw new Error(`Variable at index ${index} not found`)
    Object.assign(variable, patch)
  }

  /** Add a value label to a variable. */
  function addValueLabel(
    variableIndex: number,
    value: number | string,
    label: string,
  ) {
    const ds = dataset.value
    if (!ds) throw new Error('No dataset loaded')
    const variable = ds.variables[variableIndex]
    if (!variable) throw new Error(`Variable at index ${variableIndex} not found`)
    variable.valueLabels.push({ value, label })
  }

  /** Remove a value label from a variable by its position. */
  function removeValueLabel(variableIndex: number, labelIndex: number) {
    const ds = dataset.value
    if (!ds) throw new Error('No dataset loaded')
    const variable = ds.variables[variableIndex]
    if (!variable) throw new Error(`Variable at index ${variableIndex} not found`)
    variable.valueLabels.splice(labelIndex, 1)
  }

  /** Update a cell value by case row and variable index. */
  function updateCell(caseIndex: number, variableIndex: number, value: unknown) {
    const ds = dataset.value
    if (!ds) throw new Error('No dataset loaded')
    const variable = ds.variables[variableIndex]
    const row = ds.cases[caseIndex]
    if (!variable || !row) throw new Error('Invalid case or variable index')
    row[variable.name] = value
  }

  /** Replace all cases. */
  function setCases(cases: Dataset['cases']) {
    const ds = dataset.value
    if (!ds) throw new Error('No dataset loaded')
    ds.cases = cases
  }

  /** Clear the loaded dataset. */
  function clear() {
    dataset.value = null
    error.value = null
    loading.value = false
  }

  async function exportCsv(): Promise<string> {
    await initEngine()
    const ds = dataset.value
    if (!ds) throw new Error('No dataset loaded')
    return dataApi.exportCsv(ds)
  }

  async function exportTsv(): Promise<string> {
    await initEngine()
    const ds = dataset.value
    if (!ds) throw new Error('No dataset loaded')
    return dataApi.exportTsv(ds)
  }

  return {
    ready,
    loading,
    error,
    dataset,
    initEngine,
    loadFile,
    setVariableMetadata,
    addValueLabel,
    removeValueLabel,
    updateCell,
    setCases,
    clear,
    exportCsv,
    exportTsv,
  }
})
