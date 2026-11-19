<script setup lang="ts">
import { ref } from 'vue'
import { useDatasetStore } from '../stores/dataset'

const store = useDatasetStore()
const dragging = ref(false)
const lastFile = ref<string | null>(null)

async function handleFiles(files: FileList | null) {
  if (!files || files.length === 0) return
  const file = files[0]
  const buffer = await arrayBufferFromFile(file)
  try {
    const ds = await store.loadFile(buffer, file.name)
    lastFile.value = file.name
    return ds
  } catch (err) {
    // error is already set in the store
    throw err
  }
}

function arrayBufferFromFile(file: File): Promise<ArrayBuffer> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader()
    reader.onload = () => resolve(reader.result as ArrayBuffer)
    reader.onerror = () => reject(reader.error || new Error('Failed to read file'))
    reader.readAsArrayBuffer(file)
  })
}

function onDrop(e: DragEvent) {
  e.preventDefault()
  dragging.value = false
  handleFiles(e.dataTransfer?.files)
}

function onDragOver(e: DragEvent) {
  e.preventDefault()
  dragging.value = true
}

function onDragLeave(e: DragEvent) {
  e.preventDefault()
  dragging.value = false
}

function onClickInput(e: Event) {
  handleFiles((e.target as HTMLInputElement).files)
}
</script>

<template>
  <div class="filedrop space-y-3">
    <div
      @drop="onDrop"
      @dragover="onDragOver"
      @dragleave="onDragLeave"
      class="filedrop-zone rounded-2xl border-2 border-dashed p-8 text-center transition-all duration-200 cursor-pointer"
      :class="{
        'bg-accent-bg border-accent border-solid': dragging,
        'bg-bg border-border': !dragging,
      }"
    >
      <input
        ref="fileInput"
        type="file"
        accept=".csv,.tsv,.sav,.zsav,.html,.htm"
        @change="onClickInput"
        class="hidden"
      />
      <div class="flex flex-col items-center gap-3">
        <svg class="w-12 h-12 text-text-h" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12"></path>
        </svg>
        <p class="text-text font-medium">
          Drag and drop a file here
          <span v-if="lastFile" class="text-xs text-accent ml-2 px-2 py-0.5 rounded bg-accent/10 border border-accent/20">({{ lastFile }})</span>
        </p>
        <p class="text-xs text-text-h">
          Supported: .csv · .tsv · .sav · .zsav · .html · .htm
        </p>
        <button
          type="button"
          @click="onClickInput"
          class="text-sm text-accent hover:text-accent/80 hover:underline transition-colors"
        >
          or choose a file
        </button>
      </div>
    </div>

    <div v-if="store.loading" class="flex items-center gap-2.5 text-xs text-text-h animate-pulse">
      <span class="w-3 h-3 rounded-full border-2 border-current border-solid animate-spin"></span>
      <span>Loading dataset…</span>
    </div>

    <div v-else-if="store.error" class="rounded-xl bg-red-50 border border-red-100 p-4 text-sm text-red-700 shadow-sm">
      <div class="flex items-start gap-2">
        <svg class="w-4 h-4 flex-shrink-0 text-red-500 mt-0.5" fill="currentColor" viewBox="0 0 20 20">
          <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z" clip-rule="evenodd" />
        </svg>
        <p class="font-medium">{{ store.error }}</p>
      </div>
    </div>
  </div>
</template>
