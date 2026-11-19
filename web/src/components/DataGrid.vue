<script setup lang="ts">
import { ref } from 'vue'
import type { Dataset, Variable } from '../types/dataset'

interface Props {
  dataset: Dataset | null
}

interface Emits {
  (e: 'update-cell', caseIndex: number, variableIndex: number, value: unknown): void
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()

const editing = ref<{ caseIndex: number; variableIndex: number } | null>(null)
const editValue = ref('')

function cellValue(caseIndex: number, variable: Variable): unknown {
  return props.dataset?.cases[caseIndex]?.[variable.name]
}

function formattedValue(caseIndex: number, variable: Variable): string {
  const v = cellValue(caseIndex, variable)
  if (v === null || v === undefined) return ''
  return String(v)
}

function startEdit(caseIndex: number, variableIndex: number) {
  editing.value = { caseIndex, variableIndex }
  editValue.value = formattedValue(caseIndex, props.dataset!.variables[variableIndex])
}

function commitEdit() {
  if (!editing.value || !props.dataset) return
  const { caseIndex, variableIndex } = editing.value
  const variable = props.dataset.variables[variableIndex]
  const raw = editValue.value.trim()
  let value: unknown
  if (variable.type === 'numeric') {
    value = raw === '' ? null : Number(raw)
    if (Number.isNaN(value as number)) value = null
  } else {
    value = raw
  }
  emit('update-cell', caseIndex, variableIndex, value)
  editing.value = null
}

function cancelEdit() {
  editing.value = null
  editValue.value = ''
}

const isEditing = (caseIndex: number, variableIndex: number) =>
  editing.value?.caseIndex === caseIndex &&
  editing.value?.variableIndex === variableIndex
</script>

<template>
  <div v-if="dataset" class="data-grid overflow-auto border border-border rounded-xl shadow-sm bg-white dark:bg-bg">
    <table class="min-w-full text-sm border-collapse">
      <thead>
        <tr>
          <th class="sticky left-0 z-10 bg-bg/50 border-r border-border p-2 text-left text-xs font-semibold text-text-h uppercase tracking-wider select-none">
            #
          </th>
          <th
            v-for="(variable, vi) in dataset.variables"
            :key="variable.name"
            class="sticky top-0 z-20 bg-bg/50 border-b border-border p-2 text-left text-xs font-semibold text-text-h uppercase tracking-wider select-none min-w-[120px]"
          >
            {{ variable.name }}
          </th>
        </tr>
      </thead>
      <tbody>
        <tr
          v-for="(row, ci) in dataset.cases"
          :key="ci"
          class="border-b border-border/50 hover:bg-accent/5 transition-colors"
          :class="ci % 2 === 0 ? 'bg-white/50' : ''"
        >
          <td class="sticky left-0 z-10 bg-bg/50 border-r border-border p-2 font-mono text-xs text-text-h select-none">
            {{ ci + 1 }}
          </td>
          <td
            v-for="(variable, vi) in dataset.variables"
            :key="variable.name"
            class="p-2 border-r border-border min-w-[120px]"
          >
            <div v-if="isEditing(ci, vi)" class="relative">
              <input
                ref="editInput"
                v-model="editValue"
                class="w-full h-full p-1 bg-accent-border border-0 text-sm outline-none focus:ring-0"
                @blur="commitEdit"
                @keydown.enter="commitEdit"
                @keydown.escape="cancelEdit"
                @keydown.tab.stop.prevent="commitEdit"
              />
            </div>
            <div
              v-else
              @click="startEdit(ci, vi)"
              class="p-1 cursor-cell hover:bg-accent-bg/50 rounded transition-colors group"
              :class="variable.type === 'numeric' ? 'text-right text-text' : 'text-left text-text'"
              title="Click to edit"
            >
              {{ formattedValue(ci, variable) }}
            </div>
          </td>
        </tr>
      </tbody>
    </table>
  </div>
  <div v-else class="data-grid py-12 text-center">
    <div class="inline-flex flex-col items-center gap-3">
      <svg class="w-16 h-16 text-border" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z"></path>
      </svg>
      <p class="text-text">No dataset loaded</p>
      <p class="text-xs text-text-h">Drag and drop a CSV/TSV/.sav/.zsav/.html file to begin.</p>
    </div>
  </div>
</template>

<style scoped>
.text-text-h { color: var(--text-h); }
.cursor-cell { cursor: cell; }
</style>
