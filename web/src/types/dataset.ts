/**
 * Shared dataset data model used by the UI, Pinia store, Comlink worker,
 * and Rust/WASM engine boundary.
 *
 * All types are serialization-safe (plain objects / ArrayBuffers) so they
 * can cross the worker boundary without loss.
 */

/** Variable measurement level. */
export type Measure = "nominal" | "ordinal" | "scale";

/** SPSS variable data type. */
export type VariableType = "numeric" | "string";

/**
 * User-defined missing values.
 *
 * - `userMissing`: explicit user-missing codes (numeric or string).
 * - `systemMissing`: system-missing marker (for binary formats that track it).
 * - `null`: no missing-value definition.
 */
export type MissingValues =
  | { kind: "user"; values: (number | string)[] }
  | { kind: "system"; systemMissing: boolean }
  | null;

/** A single value -> label mapping for value labels. */
export interface ValueLabel {
  /** The coded value (numeric for numeric vars, string for string vars). */
  value: number | string;
  /** The human-readable category label. */
  label: string;
}

/** Metadata for a single SPSS variable. */
export interface Variable {
  /** Unique variable name (max 64 bytes in SAV). */
  name: string;
  /** Optional descriptive label. */
  label: string;
  /** Data type: numeric or string. */
  type: VariableType;
  /** String width in bytes (only meaningful for string variables). */
  stringWidth: number;
  /** Measurement level. */
  measure: Measure;
  /** Missing-value definition. */
  missing: MissingValues;
  /** Numeric-code -> categorical-string mappings. */
  valueLabels: ValueLabel[];
  /** Display column width (chars). */
  columnWidth: number;
  /** Cell text alignment. */
  align: "left" | "right" | "center";
}

/** A single case/record: variable name -> cell value. */
export type Case = Record<string, unknown>;

/** A complete SPSS-style dataset. */
export interface Dataset {
  variables: Variable[];
  cases: Case[];
  /** Originating file name, if loaded from disk. */
  sourceFile?: string;
}

/** Serialization-safe DTO for crossing the Comlink worker boundary. */
export interface DatasetDto {
  variables: Variable[];
  cases: Case[];
  sourceFile?: string;
}

export function isDatasetDto(value: unknown): value is DatasetDto {
  if (!value || typeof value !== "object") return false;
  const dto = value as DatasetDto;
  return Array.isArray(dto.variables) && Array.isArray(dto.cases);
}
