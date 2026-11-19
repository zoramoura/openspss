<script setup lang="ts">
import { ref, watch } from 'vue'
import type { ValueLabel, Variable } from '../types/dataset'

interface Props {
  variable: Variable
}

interface Emits {
  (e: 'update', valueLabels: ValueLabel[]): void
  (e: 'add'): void
  (e: 'remove', index: number): void
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()

// Local draft state kept in sync with the prop.
const labels = ref<ValueLabel[]>([])
watch(
  () => props.variable.valueLabels,
  (next) => {
    labels.value = next.map((vl) => ({ ...vl }))
  },
  { immediate: true },
)

function addLabel() {
  emit('add')
}

function removeLabel(index: number) {
  emit('remove', index)
}

function updateValue(index: number, field: 'value' | 'label', value: string) {
  const updated = labels.value.map((vl, i) =>
    i === index ? { ...vl, [field]: value } : vl,
  )
  labels.value = updated
  emit('update', updated)
}

function isDuplicateValue(index: number): boolean {
  const v = labels.value[index].value
  return labels.value.filter((_, i) => i !== index).some((vl) => vl.value === v)
}
</script>

<template>
  <div class="value-labels-editor space-y-3">
    <h3 class="text-sm font-medium text-text-h mb-1">
      Value labels — {{ variable.name }}
    </h3>
    <p class="text-xs text-text mb-2">
      Map numeric codes to category labels (numeric variables only).
    </p>
    <div class="overflow-auto border border-border rounded-md">
      <table class="min-w-full text-sm">
        <thead>
          <tr class="bg-bg border-b border-border">
            <th class="p-2 text-left text-text font-medium">Value</th>
            <th class="p-2 text-left text-text font-medium">Label</th>
            <th class="p-2 text-left text-text font-medium w-16">Actions</th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="(vl, i) in labels"
            :key="i"
            class="border-b border-border last:border-0"
          >
            <td class="p-2">
              <input
                :value="vl.value"
                class="w-full bg-bg border-0 p-1 outline-none text-sm"
                :class="isDuplicateValue(i) ? 'bg-red-50' : ''"
                @input="updateValue(i, 'value', $event.target.value)"
              />
            </td>
            <td class="p-2">
              <input
                :value="vl.label"
                class="w-full bg-bg border-0 p-1 outline-none text-sm"
                @input="updateValue(i, 'label', $event.target.value)"
              />
            </td>
            <td class="p-2">
              <button
                type="button"
                @click="removeLabel(i)"
                class="text-xs text-accent hover:underline"
                :disabled="labels.length === 1"
              >
                Remove
              </button>
            </td>
          </tr>
          <tr v-if="labels.length === 0">
            <td colspan="3" class="p-3 text-text text-center">
              No value labels defined.
            </td>
          </tr>
        </tbody>
      </table>
    </div>
    <button
      type="button"
      @click="addLabel"
      class="text-sm text-accent hover:underline flex items-center gap-1"
    >
      <span>+</span> Add value label
    </button>
  </div>
</template>
