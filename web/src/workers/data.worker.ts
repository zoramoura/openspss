import * as Comlink from 'comlink'
import init, { read_sav, read_zsav } from '../wasm/engine'

/** Shape returned by the Rust SAV/ZSAV reader (serialised as JSON). */
interface RawVariableDto {
  name: string
  label: string
  var_type: 'numeric' | 'string'
  string_width: number
  measure: string
  missing: RawMissingDto | null
  value_labels: RawValueLabelDto[]
  column_width: number
  align: string
}

interface RawValueLabelDto {
  value: string
  label: string
}

interface RawMissingDto {
  kind: 'user' | 'system'
  content: (number | string)[] | { system_missing: boolean }
}

interface RawDatasetDto {
  variables: RawVariableDto[]
  cases: string[][]
  source_file?: string
}

/** Public types consumed by the store/UI (mirrors web/src/types/dataset.ts). */
export type Measure = 'nominal' | 'ordinal' | 'scale'
export type VariableType = 'numeric' | 'string'

export interface ValueLabel {
  value: number | string
  label: string
}

export interface MissingValues {
  kind: 'user' | 'system'
  values?: (number | string)[]
  system_missing?: boolean
}

export interface Variable {
  name: string
  label: string
  type: VariableType
  stringWidth: number
  measure: Measure
  missing: MissingValues | null
  valueLabels: ValueLabel[]
  columnWidth: number
  align: 'left' | 'right' | 'center'
}

export interface Case {
  [name: string]: unknown
}

export interface Dataset {
  variables: Variable[]
  cases: Case[]
  sourceFile?: string
}

function convertMissing(raw: RawMissingDto | null): MissingValues | null {
  if (raw === null) return null
  if (raw.kind === 'system') {
    return {
      kind: 'system',
      system_missing: Boolean(
        (raw.content as { system_missing?: boolean })?.system_missing,
      ),
    }
  }
  return {
    kind: 'user',
    values: Array.isArray(raw.content)
      ? (raw.content as (number | string)[])
      : [],
  }
}

function convertValueLabel(
  raw: RawValueLabelDto,
  varType: VariableType,
): ValueLabel {
  const value: number | string =
    varType === 'numeric' ? Number(raw.value) : raw.value
  return { value, label: raw.label }
}

function convertVariable(raw: RawVariableDto): Variable {
  const varType: VariableType = raw.var_type
  return {
    name: raw.name,
    label: raw.label,
    type: varType,
    stringWidth: Number(raw.string_width) || 0,
    measure: (['nominal', 'ordinal', 'scale'].includes(raw.measure)
      ? raw.measure
      : 'nominal') as Measure,
    missing: convertMissing(raw.missing),
    valueLabels: (raw.value_labels ?? []).map((vl) =>
      convertValueLabel(vl, varType),
    ),
    columnWidth: Number(raw.column_width) || 12,
    align: ['left', 'right', 'center'].includes(raw.align)
      ? raw.align
      : 'left',
  }
}

function convertCase(row: string[], variables: Variable[]): Case {
  const record: Case = {}
  variables.forEach((v, i) => {
    const raw = row[i] ?? ''
    if (v.type === 'numeric') {
      record[v.name] = raw === '' ? null : Number(raw)
    } else {
      record[v.name] = raw
    }
  })
  return record
}

function convertDataset(raw: RawDatasetDto): Dataset {
  const variables = raw.variables.map(convertVariable)
  const cases = raw.cases.map((row) => convertCase(row, variables))
  return {
    variables,
    cases,
    sourceFile: raw.source_file,
  }
}

// ---------------------------------------------------------------------------
// Delimited text (CSV / TSV) parsing
// ---------------------------------------------------------------------------

type Row = string[]

function splitLines(text: string): string[] {
  return text.split(/\r\n|\r|\n/)
}

/**
 * Parse a single delimited line respecting double-quoted fields
 * (RFC 4180-compatible). `""` inside a quoted field is an escaped quote.
 */
function parseDelimitedLine(line: string, delimiter: string): Row {
  const fields: string[] = []
  let i = 0
  const len = line.length
  while (i <= len) {
    if (i === len) {
      fields.push('')
      break
    }
    let field = ''
    if (line[i] === '"') {
      // Quoted field.
      i++ // skip opening quote
      while (i < len) {
        const c = line[i]
        if (c === '"') {
          if (i + 1 < len && line[i + 1] === '"') {
            field += '"'
            i += 2
          } else {
            i++ // skip closing quote
            break
          }
        } else {
          field += c
          i++
        }
      }
      // Skip the delimiter after the closing quote, if present.
      if (line.startsWith(delimiter, i)) {
        i += delimiter.length
      }
    } else {
      const end = line.indexOf(delimiter, i)
      if (end === -1) {
        field = line.slice(i)
        i = len
      } else {
        field = line.slice(i, end)
        i = end + delimiter.length
      }
    }
    fields.push(field)
    if (i > len) break
  }
  return fields
}

/**
 * Detect whether a column is numeric: every non-empty cell parses as a number.
 */
function isNumericColumn(rows: Row[], colIndex: number): boolean {
  let hasValue = false
  for (let r = 1; r < rows.length; r++) {
    const cell = rows[r][colIndex]
    if (cell === undefined || cell === '') continue
    hasValue = true
    if (Number.isNaN(Number(cell))) return false
  }
  return hasValue
}

function parseDelimited(text: string, delimiter: string): Dataset {
  const lines = splitLines(text).filter((l) => l.length > 0)
  if (lines.length === 0) {
    return { variables: [], cases: [], sourceFile: undefined }
  }

  const rows: Row[] = lines.map((line) => parseDelimitedLine(line, delimiter))
  const header = rows[0]
  const dataRows = rows.slice(1)

  const variables: Variable[] = header.map((name, i) => {
    const varType: VariableType = isNumericColumn(dataRows, i)
      ? 'numeric'
      : 'string'
    return {
      name: name === '' ? `V${i + 1}` : name,
      label: '',
      type: varType,
      stringWidth: varType === 'string' ? 255 : 0,
      measure: 'nominal',
      missing: null,
      valueLabels: [],
      columnWidth: varType === 'string' ? Math.max(name.length + 2, 8) : 12,
      align: varType === 'string' ? 'left' : 'right',
    }
  })

  const cases: Case[] = dataRows.map((row) => convertCase(row, variables))

  return { variables, cases, sourceFile: undefined }
}

function parseCsv(text: string): Dataset {
  return parseDelimited(text, ',')
}

function parseTsv(text: string): Dataset {
  return parseDelimited(text, '\t')
}

// ---------------------------------------------------------------------------
// HTML table parsing (best-effort, no DOM dependency)
// ---------------------------------------------------------------------------

function unescapeHtml(text: string): string {
  if (typeof document === 'undefined') {
    // Minimal entity fallback for worker environments without a DOM.
    return text
      .replace(/&amp;/g, '&')
      .replace(/&lt;/g, '<')
      .replace(/&gt;/g, '>')
      .replace(/&quot;/g, '"')
      .replace(/&#39;/g, "'")
  }
  const el = document.createElement('div')
  el.innerHTML = text
  return el.textContent ?? ''
}

function parseHtml(text: string): Dataset {
  // Very lightweight <table> extraction. Falls back gracefully if no table.
  const tableMatch = text.match(/<table[\s\S]*?<\/table>/i)
  if (!tableMatch) {
    return { variables: [], cases: [], sourceFile: undefined }
  }
  const table = tableMatch[0]

  const rowMatches = table.match(/<tr[\s\S]*?<\/tr>/gi)
  if (!rowMatches) {
    return { variables: [], cases: [], sourceFile: undefined }
  }

  const rows: Row[] = rowMatches.map((row) => {
    const cells = row.match(/<t[hd][\s\S]*?<\/t[hd]>/gi)
    if (!cells) return []
    return cells.map((cell) => {
      const inner = cell.replace(/<t[hd][^>]*>/i, '').replace(/<\/t[hd]>/i, '')
      return unescapeHtml(inner).trim()
    })
  })

  if (rows.length === 0) {
    return { variables: [], cases: [], sourceFile: undefined }
  }

  const header = rows[0]
  const dataRows = rows.slice(1)
  const variables: Variable[] = header.map((name, i) => ({
    name: name === '' ? `V${i + 1}` : name,
    label: '',
    type: 'string',
    stringWidth: 255,
    measure: 'nominal',
    missing: null,
    valueLabels: [],
    columnWidth: Math.max(name.length + 2, 8),
    align: 'left',
  }))

  const cases: Case[] = dataRows.map((row) => convertCase(row, variables))
  return { variables, cases, sourceFile: undefined }
}

// ---------------------------------------------------------------------------
// Export
// ---------------------------------------------------------------------------

function escapeCsvField(value: unknown, delimiter: string): string {
  const str = value === null || value === undefined ? '' : String(value)
  const needsQuotes =
    str.includes(delimiter) ||
    str.includes('"') ||
    str.includes('\n') ||
    str.includes('\r')
  if (!needsQuotes) return str
  return `"${str.replace(/"/g, '""')}"`
}

function datasetToDelimited(dataset: Dataset, delimiter: string): string {
  const lines: string[] = []
  const header = dataset.variables.map((v) => v.name).join(delimiter)
  lines.push(header)
  for (const row of dataset.cases) {
    lines.push(
      dataset.variables
        .map((v) => escapeCsvField(row[v.name], delimiter))
        .join(delimiter),
    )
  }
  return lines.join('\n')
}

function datasetToCsv(dataset: Dataset): string {
  return datasetToDelimited(dataset, ',')
}

function datasetToTsv(dataset: Dataset): string {
  return datasetToDelimited(dataset, '\t')
}

// ---------------------------------------------------------------------------
// Comlink API
// ---------------------------------------------------------------------------

let engineReady = false
let engineInit: Promise<void> | undefined

async function ensureEngine() {
  if (engineReady) return
  engineInit ??= init().then(() => {
    engineReady = true
  })
  await engineInit
}

const dataApi = {
  async initEngine() {
    await ensureEngine()
  },
  /**
   * Read a dropped file into a Dataset. `fileName` determines the parser
   * (.csv/.tsv/.sav/.zsav/.html/.htm). `buffer` is the raw file bytes.
   */
  async readFile(buffer: ArrayBuffer, fileName: string): Promise<Dataset> {
    const name = fileName.toLowerCase()
    const bytes = new Uint8Array(buffer)

    if (name.endsWith('.sav')) {
      await ensureEngine()
      const json = read_sav(bytes)
      const raw: RawDatasetDto = JSON.parse(json)
      return convertDataset(raw)
    }
    if (name.endsWith('.zsav')) {
      await ensureEngine()
      const json = read_zsav(bytes)
      const raw: RawDatasetDto = JSON.parse(json)
      return convertDataset(raw)
    }
    if (name.endsWith('.csv')) {
      const text = new TextDecoder().decode(bytes)
      return parseCsv(text)
    }
    if (name.endsWith('.tsv')) {
      const text = new TextDecoder().decode(bytes)
      return parseTsv(text)
    }
    if (name.endsWith('.html') || name.endsWith('.htm')) {
      const text = new TextDecoder().decode(bytes)
      return parseHtml(text)
    }
    throw new Error(
      `Unsupported file type: "${fileName}". Use .csv, .tsv, .sav, .zsav, .html or .htm`,
    )
  },
  async parseCsv(buffer: ArrayBuffer): Promise<Dataset> {
    const text = new TextDecoder().decode(buffer)
    return parseCsv(text)
  },
  async parseTsv(buffer: ArrayBuffer): Promise<Dataset> {
    const text = new TextDecoder().decode(buffer)
    return parseTsv(text)
  },
  async parseHtml(buffer: ArrayBuffer): Promise<Dataset> {
    const text = new TextDecoder().decode(buffer)
    return parseHtml(text)
  },
  async exportCsv(dataset: Dataset): Promise<string> {
    return datasetToCsv(dataset)
  },
  async exportTsv(dataset: Dataset): Promise<string> {
    return datasetToTsv(dataset)
  },
}

export type DataWorkerApi = typeof dataApi
Comlink.expose(dataApi)
