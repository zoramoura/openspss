<script setup lang="ts">
import { ref, computed } from 'vue'
import { useDatasetStore } from './stores/dataset'
import FileDrop from './components/FileDrop.vue'
import DataGrid from './components/DataGrid.vue'
import VariableView from './components/VariableView.vue'

const store = useDatasetStore()
const activeTab = ref<'data' | 'variable'>('data')

const dataset = computed(() => store.dataset)

async function downloadDataset(format: 'csv' | 'tsv') {
  const ds = dataset.value
  if (!ds) return
  try {
    const text = format === 'csv' ? await store.exportCsv() : await store.exportTsv()
    const blob = new Blob([text], { type: 'text/plain;charset=utf-8' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    const base = (ds.sourceFile || 'dataset').replace(/\.[^/.]+$/, '')
    a.href = url
    a.download = `${base}.${format === 'csv' ? 'csv' : 'tsv'}`
    document.body.appendChild(a)
    a.click()
    document.body.removeChild(a)
    URL.revokeObjectURL(url)
  } catch (err) {
    // error already reflected in store.error
  }
}

function clearDataset() {
  store.clear()
}
</script>

<template>
  <div class="app flex flex-col h-screen overflow-hidden">
    <header class="border-b border-border bg-bg p-4 flex flex-col gap-3">
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-2">
          <span class="font-semibold text-text-h">OpenSPSS</span>
          <span class="text-xs text-text px-2 py-0.5 rounded bg-border border border-border">Data Core</span>
        </div>
        <FileDrop />
      </div>

      <div class="flex items-center gap-2">
        <button
          type="button"
          :class="[
            'px-3 py-1.5 rounded text-sm transition-colors',
            activeTab === 'data'
              ? 'bg-accent text-white'
              : 'bg-border text-text border border-border hover:bg-accent/50',
          ]"
          @click="activeTab = 'data'"
        >
          Data View
        </button>
        <button
          type="button"
          :class="[
            'px-3 py-1.5 rounded text-sm transition-colors',
            activeTab === 'variable'
              ? 'bg-accent text-white'
              : 'bg-border text-text border border-border hover:bg-accent/50',
          ]"
          @click="activeTab = 'variable'"
        >
          Variable View
        </button>

        <div class="flex-1"></div>

        <div v-if="dataset" class="flex items-center gap-3 text-xs text-text">
          <span>{{ dataset.variables.length }} variables · {{ dataset.cases.length }} cases</span>
          <button
            type="button"
            @click="downloadDataset('csv')"
            class="px-3 py-1.5 rounded text-sm border border-border bg-bg hover:bg-accent/50 transition-colors"
          >
            Export CSV
          </button>
          <button
            type="button"
            @click="downloadDataset('tsv')"
            class="px-3 py-1.5 rounded text-sm border border-border bg-bg hover:bg-accent/50 transition-colors"
          >
            Export TSV
          </button>
          <button
            type="button"
            @click="clearDataset"
            class="px-3 py-1.5 rounded text-sm text-red-600 hover:bg-red-50 transition-colors"
          >
            Clear
          </button>
        </div>
      </div>

      <div v-if="store.error" class="rounded-md bg-red-50 border border-red-200 p-3 text-xs text-red-800">
        {{ store.error }}
      </div>
    </header>

    <main class="flex-1 overflow-auto p-4">
      <template v-if="activeTab === 'data'">
        <DataGrid :dataset="dataset" @update-cell="store.updateCell" />
      </template>
      <template v-else>
        <VariableView />
      </template>
    </main>

    <footer class="border-t border-border p-3 text-xs text-text flex justify-between">
      <span>
        {{ store.loading ? 'Loading…' : store.dataset ? 'Dataset loaded' : 'No dataset loaded' }}
      </span>
      <span class="text-accent">Engine: {{ store.ready ? 'ready' : 'initializing' }}</span>
    </footer>
  </div>
</template>
