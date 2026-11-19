<script setup lang="ts">
import { computed, ref } from 'vue'
import { useDatasetStore } from '../stores/dataset'
import type { MissingValues, Variable } from '../types/dataset'
import ValueLabelsEditor from './ValueLabelsEditor.vue'

const store = useDatasetStore()
const dataset = computed(() => store.dataset)
const selectedIndex = ref<number | null>(null)
const newLabelValue = ref('')
const newLabelLabel = ref('')

const measures = ['nominal', 'ordinal', 'scale'] as const

function select(index: number) {
  selectedIndex.value = index
}

function isSelected(index: number) {
  return selectedIndex.value === index
}

function updateVariable(index: number, patch: Partial<Variable>) {
  store.setVariableMetadata(index, patch)
}

function setMissing(index: number, missing: MissingValues | null) {
  store.setVariableMetadata(index, { missing })
}

function addValueLabel(variableIndex: number, value: string, label: string) {
  const variable = dataset.value?.variables[variableIndex]
  if (!variable) return
  const parsed: number | string =
    variable.type === 'numeric' ? Number(value) : value
  store.addValueLabel(variableIndex, parsed, label)
}

function removeValueLabel(variableIndex: number, labelIndex: number) {
  store.removeValueLabel(variableIndex, labelIndex)
}

function updateValueLabels(
  variableIndex: number,
  valueLabels: { value: number | string; label: string }[],
) {
  const variable = dataset.value?.variables[variableIndex]
  if (!variable) return
  const normalized = valueLabels.map((vl) => ({
    value: variable.type === 'numeric' ? Number(vl.value) : vl.value,
    label: vl.label,
  }))
  variable.valueLabels = normalized
}

const selectedVariable = computed(() =>
  selectedIndex.value === null || !dataset.value
    ? null
    : dataset.value.variables[selectedIndex.value],
)

function missingBadgeClass(missing: MissingValues | null): string {
  if (!missing) return 'bg-bg'
  return missing.kind === 'user'
    ? 'bg-amber-50 border-amber-200 text-amber-800'
    : 'bg-blue-50 border-blue-200 text-blue-800'
}

function missingLabel(missing: MissingValues | null): string {
  if (!missing) return 'None'
  if (missing.kind === 'system') return 'System'
  return `User (${missing.values.length})`
}
</script>

<template>
  <div class="variable-view space-y-5">
    <div class="flex items-center gap-2 border-b border-border pb-3">
      <h2 class="text-base font-semibold text-text-h">Variable View</h2>
      <span
        v-if="dataset"
        class="ml-auto text-xs text-text-h px-2.5 py-1 rounded bg-bg border border-border"
      >
        {{ dataset.variables.length }} variables
      </span>
    </div>

    <div v-if="!dataset" class="py-12 text-center">
      <div class="inline-flex flex-col items-center gap-2">
        <svg class="w-10 h-10 text-border" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"></path>
        </svg>
        <p class="text-text">No dataset loaded</p>
      </div>
    </div>

    <div v-else class="overflow-auto border border-border rounded-xl shadow-sm">
      <table class="min-w-full text-sm border-collapse">
        <thead class="bg-bg/50">
          <tr>
            <th class="sticky left-0 z-10 bg-bg/50 border-r border-border p-3 text-left text-xs font-semibold text-text-h uppercase tracking-wider select-none w-16">#</th>
            <th class="p-3 text-left text-xs font-semibold text-text-h uppercase tracking-wider">Name</th>
            <th class="p-3 text-left text-xs font-semibold text-text-h uppercase tracking-wider">Label</th>
            <th class="p-3 text-left text-xs font-semibold text-text-h uppercase tracking-wider w-32">Type</th>
            <th class="p-3 text-left text-xs font-semibold text-text-h uppercase tracking-wider w-32">Measure</th>
            <th class="p-3 text-left text-xs font-semibold text-text-h uppercase tracking-wider w-40">Missing</th>
            <th class="p-3 text-left text-xs font-semibold text-text-h uppercase tracking-wider w-24">Align</th>
          </tr>
        </thead>
        <tbody class="divide-y divide-border/50">
          <tr
            v-for="(variable, index) in dataset.variables"
            :key="variable.name"
            class="hover:bg-accent/5 transition-colors"
            :class="isSelected(index) ? 'bg-accent/10' : ''"
          >
            <td class="sticky left-0 z-10 bg-white dark:bg-bg border-r border-border p-3 font-mono text-xs text-text-h select-none cursor-pointer" @click="select(index)">{{ index + 1 }}</td>
            <td class="p-3 text-left text-text cursor-pointer" @click="select(index)">{{ variable.name }}</td>
            <td class="p-3 text-left text-text cursor-pointer" @click="select(index)">{{ variable.label || '—' }}</td>
            <td class="p-3 text-left text-text cursor-pointer" @click="select(index)">
              <span class="inline-flex items-center gap-1 px-2 py-0.5 rounded text-xs font-medium"
                :class="variable.type === 'numeric' ? 'bg-blue-50 text-blue-700 border border-blue-100' : 'bg-amber-50 text-amber-700 border border-amber-100'"
              >
                {{ variable.type }}
              </span>
            </td>
            <td class="p-3 text-left text-text cursor-pointer" @click="select(index)">
              <span class="capitalize text-xs">{{ variable.measure }}</span>
            </td>
            <td class="p-3 text-left text-text cursor-pointer" @click="select(index)">
              <span
                class="inline-flex items-center gap-1 px-2 py-0.5 rounded text-xs font-medium"
                :class="missingBadgeClass(variable.missing)"
              >
                {{ missingLabel(variable.missing) }}
              </span>
            </td>
            <td class="p-3 text-left text-text cursor-pointer" @click="select(index)">
              <span class="capitalize text-xs">{{ variable.align }}</span>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <div v-if="selectedVariable" class="mt-5 space-y-4">
      <h3 class="text-sm font-semibold text-text-h">
        {{ selectedVariable.name }} — details
      </h3>

      <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
        <div>
          <label class="block text-xs text-text-h mb-1">Name</label>
          <input
            v-model="selectedVariable.name"
            @input="updateVariable(selectedIndex!, { name: selectedVariable.name })"
            class="w-full bg-bg border border-border p-2 rounded text-sm"
          />
        </div>
        <div>
          <label class="block text-xs text-text-h mb-1">Label</label>
          <input
            v-model="selectedVariable.label"
            @input="updateVariable(selectedIndex!, { label: selectedVariable.label })"
            class="w-full bg-bg border border-border p-2 rounded text-sm"
          />
        </div>
        <div>
          <label class="block text-xs text-text-h mb-1">String width (bytes)</label>
          <input
            v-model.number="selectedVariable.stringWidth"
            @change="updateVariable(selectedIndex!, { stringWidth: selectedVariable.stringWidth })"
            class="w-full bg-bg border border-border p-2 rounded text-sm"
            :disabled="selectedVariable.type !== 'string'"
          />
        </div>
        <div>
          <label class="block text-xs text-text-h mb-1">Column width (chars)</label>
          <input
            v-model.number="selectedVariable.columnWidth"
            @change="updateVariable(selectedIndex!, { columnWidth: selectedVariable.columnWidth })"
            class="w-full bg-bg border border-border p-2 rounded text-sm"
          />
        </div>
        <div>
          <label class="block text-xs text-text-h mb-1">Alignment</label>
          <select
            v-model="selectedVariable.align"
            @change="updateVariable(selectedIndex!, { align: selectedVariable.align })"
            class="w-full bg-bg border border-border p-2 rounded text-sm"
          >
            <option value="left">Left</option>
            <option value="right">Right</option>
            <option value="center">Center</option>
          </select>
        </div>
      </div>

      <!-- Missing values editor -->
      <div class="border border-border rounded-md p-3">
        <h4 class="text-xs font-semibold text-text-h mb-2">Missing values</h4>
        <div class="space-y-2">
          <label class="flex items-center gap-2 text-sm">
            <input
              type="checkbox"
              :checked="selectedVariable.missing?.kind === 'system'"
              @change="(e) => {
                const missing: MissingValues | null = e.target.checked
                  ? { kind: 'system', system_missing: true }
                  : selectedVariable.missing?.kind === 'user' && !e.target.checked
                    ? null
                    : selectedVariable.missing
                setMissing(selectedIndex!, missing)
              }"
            />
            System missing
          </label>
          <div v-if="selectedVariable.missing?.kind === 'user'" class="ml-6 space-y-1">
            <label class="text-xs text-text">User-missing values (comma-separated)</label>
            <input
              :value="selectedVariable.missing.values.join(', ')"
              @input="(e) => {
                const raw = e.target.value.split(',').map(s => s.trim()).filter(Boolean)
                setMissing(selectedIndex!, { kind: 'user', values: raw })
              }"
              class="w-full bg-bg border border-border p-2 rounded text-sm"
            />
          </div>
          <div v-else-if="selectedVariable.missing?.kind === 'system'">
            <p class="text-xs text-text">System missing is enabled.</p>
          </div>
          <div v-else class="ml-6">
            <button
              type="button"
              @click="() => setMissing(selectedIndex!, { kind: 'user', values: [] })"
              class="text-xs text-accent hover:text-accent/80 hover:underline transition-colors"
            >
              Add user-missing values
            </button>
          </div>
        </div>
      </div>

      <!-- Value labels editor -->
      <ValueLabelsEditor
        :variable="selectedVariable"
        @update="(vls) => updateValueLabels(selectedIndex!, vls)"
        @remove="(idx) => removeValueLabel(selectedIndex!, idx)"
      />
      <div class="flex gap-2 items-end">
        <label class="text-xs text-text">New value</label>
        <input
          v-model="newLabelValue"
          class="w-24 bg-bg border border-border p-2 rounded text-sm"
          placeholder="code"
        />
        <label class="text-xs text-text">Label</label>
        <input
          v-model="newLabelLabel"
          class="w-32 bg-bg border border-border p-2 rounded text-sm"
          placeholder="category"
        />
        <button
          type="button"
          @click="() => {
            if (selectedIndex === null) return
            addValueLabel(selectedIndex, newLabelValue, newLabelLabel)
            newLabelValue = ''
            newLabelLabel = ''
          }"
          class="flex items-center gap-1.5 px-3 py-1.5 rounded bg-accent text-white text-xs font-medium hover:bg-accent/90 transition-colors"
        >
          Add
        </button>
      </div>
    </div>
  </div>
</template>






