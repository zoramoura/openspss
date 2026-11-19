use std::io::{Cursor, Read};

use serde::Serialize;

// ---------------------------------------------------------------------------
// Data structures mirrored on the JS/TS side (see web/src/types/dataset.ts).
// Serialized to JSON and passed across the wasm/comlink boundary as a String.
// ---------------------------------------------------------------------------

#[derive(Serialize, Debug, Clone)]
pub struct ValueLabelDto {
    pub value: String,
    pub label: String,
}

#[derive(Serialize, Debug, Clone)]
#[serde(tag = "kind", content = "content")]
pub enum MissingDto {
    #[serde(rename = "user")]
    User { values: Vec<String> },
    #[serde(rename = "system")]
    System { system_missing: bool },
}

#[derive(Serialize, Debug, Clone)]
pub struct VariableDto {
    pub name: String,
    pub label: String,
    pub var_type: String,      // "numeric" | "string"
    pub string_width: u32,
    pub measure: String,       // "nominal" | "ordinal" | "scale"
    pub missing: Option<MissingDto>,
    pub value_labels: Vec<ValueLabelDto>,
    pub column_width: u32,
    pub align: String,         // "left" | "right" | "center"
}

#[derive(Serialize, Debug, Clone)]
pub struct DatasetDto {
    pub variables: Vec<VariableDto>,
    pub cases: Vec<Vec<String>>, // row-major, aligned with `variables` order
    pub source_file: Option<String>,
}

// ---------------------------------------------------------------------------
// Endianness-aware byte reader
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq)]
enum Endian {
    Big,
    Little,
}

struct Reader<'a> {
    cursor: Cursor<&'a [u8]>,
    endian: Endian,
}

impl<'a> Reader<'a> {
    fn new(bytes: &'a [u8], endian: Endian) -> Self {
        Self {
            cursor: Cursor::new(bytes),
            endian,
        }
    }

    fn endian(&self) -> Endian {
        self.endian
    }

    /// Number of bytes remaining unread.
    fn remaining(&self) -> usize {
        let pos = self.cursor.position() as usize;
        self.cursor.get_ref().len().saturating_sub(pos)
    }

    /// Read exactly `n` bytes, advancing the cursor.
    fn take(&mut self, n: usize) -> Result<Vec<u8>, String> {
        let pos = self.cursor.position() as usize;
        let end = pos.checked_add(n).ok_or("integer overflow")?;
        let buf = self.cursor.get_ref();
        if end > buf.len() {
            return Err(format!(
                "unexpected end of file: needed {} bytes, {} remain",
                n,
                self.remaining()
            ));
        }
        let mut out = vec![0u8; n];
        self.cursor.read_exact(&mut out).map_err(|e| e.to_string())?;
        Ok(out)
    }

    fn read_i32(&mut self) -> Result<i32, String> {
        let b = self.take(4)?;
        match self.endian {
            Endian::Big => Ok(i32::from_be_bytes([b[0], b[1], b[2], b[3]])),
            Endian::Little => Ok(i32::from_le_bytes([b[0], b[1], b[2], b[3]])),
        }
    }

    fn read_u32(&mut self) -> Result<u32, String> {
        let b = self.take(4)?;
        match self.endian {
            Endian::Big => Ok(u32::from_be_bytes([b[0], b[1], b[2], b[3]])),
            Endian::Little => Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]])),
        }
    }

    fn read_f64(&mut self) -> Result<f64, String> {
        let b = self.take(8)?;
        match self.endian {
            Endian::Big => Ok(f64::from_be_bytes([
                b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7],
            ])),
            Endian::Little => Ok(f64::from_le_bytes([
                b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7],
            ])),
        }
    }

    /// Read a null-padded fixed-width string and trim trailing NUL/spaces.
    fn read_fixed_string(&mut self, n: usize) -> Result<String, String> {
        let b = self.take(n)?;
        let end = b
            .iter()
            .rposition(|&c| c != 0 && c != b' ')
            .map(|i| i + 1)
            .unwrap_or(0);
        String::from_utf8(b[..end].to_vec())
            .map_err(|e| format!("invalid UTF-8 in fixed string: {}", e))
    }
}

/// The SPSS SAV "variable dictionary" record (record type 2) fixed header.
///
/// Layout (all multi-byte integers in the file's native endianness):
///   bias          i32  = 100 uncompressed; otherwise compressed
///   n_vars        i32
///   n_cases       i32
///   name_len      i32  = 8
///   label_len     i32
///   vallabel_len  i32
///   missing_len   i32
///   format_len    i32
///   measure_len   i32
struct SavHeader {
    n_vars: i32,
    n_cases: i32,
    bias: i32,
    name_len: i32,
    label_len: i32,
    vallabel_len: i32,
    missing_len: i32,
    format_len: i32,
    measure_len: i32,
}

fn read_header(r: &mut Reader) -> Result<SavHeader, String> {
    let bias = r.read_i32()?;
    if bias != 100 {
        return Err(format!(
            "compressed SAV data (bias = {}) is not supported by this reader; \
             use an uncompressed .sav or a .zsav file",
            bias
        ));
    }
    let n_vars = r.read_i32()?;
    let n_cases = r.read_i32()?;
    let name_len = r.read_i32()?;
    let label_len = r.read_i32()?;
    let vallabel_len = r.read_i32()?;
    let missing_len = r.read_i32()?;
    let format_len = r.read_i32()?;
    let measure_len = r.read_i32()?;

    if n_vars < 0 {
        return Err(format!("invalid variable count: {}", n_vars));
    }
    if name_len != 8 {
        return Err(format!("unsupported variable name entry length: {}", name_len));
    }

    Ok(SavHeader {
        n_vars,
        n_cases,
        bias,
        name_len,
        label_len,
        vallabel_len,
        missing_len,
        format_len,
        measure_len,
    })
}

fn read_variable_names(r: &mut Reader, n_vars: i32) -> Result<Vec<String>, String> {
    let mut names = Vec::with_capacity(n_vars as usize);
    for _ in 0..n_vars as usize {
        let raw = r.read_fixed_string(8)?;
        // SPSS reserves the '$' prefix historically; ignore it for naming.
        let name = raw.trim_start_matches('$');
        if name.is_empty() {
            return Err("encountered an empty variable name".to_string());
        }
        names.push(name.to_string());
    }
    Ok(names)
}

fn read_variable_labels(r: &mut Reader, n_vars: i32, label_len: i32) -> Result<Vec<String>, String> {
    if label_len <= 0 {
        return Ok(vec![String::new(); n_vars as usize]);
    }
    let mut labels = Vec::with_capacity(n_vars as usize);
    for _ in 0..n_vars as usize {
        labels.push(r.read_fixed_string(label_len as usize)?);
    }
    Ok(labels)
}

/// Map an SPSS print/write format to (variable type, string width).
///
/// Numeric formats start with a letter such as F, E, etc. String formats
/// start with 'A' followed by the width (e.g. "A12"). When the format block
/// cannot be interpreted we fall back to numeric / width 8.
fn parse_format(text: &str) -> (String, u32) {
    let t = text.trim();
    if let Some(rest) = t.strip_prefix('A') {
        let width: u32 = rest
            .split(char::is_alphabetic)
            .next()
            .unwrap_or(rest)
            .trim()
            .parse()
            .unwrap_or(0);
        return ("string".to_string(), width);
    }
    ("numeric".to_string(), 8)
}

fn read_formats(
    r: &mut Reader,
    n_vars: i32,
    format_len: i32,
) -> Result<Vec<(String, u32)>, String> {
    if format_len <= 0 {
        return Ok(vec![("numeric".to_string(), 8); n_vars as usize]);
    }
    let mut out = Vec::with_capacity(n_vars as usize);
    for _ in 0..n_vars as usize {
        let raw = r.read_fixed_string(format_len as usize)?;
        out.push(parse_format(&raw));
    }
    Ok(out)
}

/// Read measure levels. Each measure entry is a small integer code:
///   1 = nominal, 2 = ordinal, 3 = scale (SPSS convention).
fn read_measures(
    r: &mut Reader,
    n_vars: i32,
    measure_len: i32,
) -> Result<Vec<String>, String> {
    if measure_len <= 0 {
        return Ok(vec!["nominal".to_string(); n_vars as usize]);
    }
    let mut out = Vec::with_capacity(n_vars as usize);
    for _ in 0..n_vars as usize {
        let raw = r.read_fixed_string(measure_len as usize)?;
        let code: i32 = raw.trim().parse().unwrap_or(1);
        let measure = match code {
            2 => "ordinal",
            3 => "scale",
            _ => "nominal",
        };
        out.push(measure.to_string());
    }
    Ok(out)
}

/// Skip a block of `n_vars * entry_len` bytes (used for value-label and
/// missing-value sections whose precise sub-record layout we do not attempt
/// to decode). Returns the raw bytes so a future version can parse them.
fn skip_block(r: &mut Reader, n_vars: i32, entry_len: i32) -> Result<Vec<u8>, String> {
    if entry_len <= 0 {
        return Ok(Vec::new());
    }
    let total = (n_vars as usize).checked_mul(entry_len as usize).ok_or("overflow")?;
    r.take(total)
}

/// Interpret an 8-byte IEEE-754 double given endianness.
fn f64_from_bytes(bytes: &[u8; 8], endian: Endian) -> f64 {
    match endian {
        Endian::Big => f64::from_be_bytes(*bytes),
        Endian::Little => f64::from_le_bytes(*bytes),
    }
}

/// Read uncompressed case data. Each case is a fixed-length record; each
/// variable occupies 8 bytes (float64) if numeric, or `string_width` bytes
/// if string. Values are packed consecutively with no per-case header.
fn read_data_uncompressed(
    r: &mut Reader,
    variables: &[VariableDto],
    n_cases: i32,
) -> Result<Vec<Vec<String>>, String> {
    let record_size: usize = variables
        .iter()
        .map(|v| {
            if v.var_type == "string" {
                v.string_width as usize
            } else {
                8
            }
        })
        .sum();
    if record_size == 0 {
        return Ok(vec![Vec::new(); n_cases as usize]);
    }

    let endian = r.endian();
    let mut cases = Vec::with_capacity(n_cases as usize);
    for _ in 0..n_cases as usize {
        let mut row = Vec::with_capacity(variables.len());
        for v in variables {
            let width = if v.var_type == "string" {
                v.string_width as usize
            } else {
                8
            };
            let b = r.take(width)?;
            if v.var_type == "string" {
                let s = String::from_utf8_lossy(&b)
                    .trim_end_matches(|c| c == '\0' || c == ' ')
                    .to_string();
                row.push(s);
            } else {
                if b.len() < 8 {
                    row.push(String::new());
                    continue;
                }
                let arr: [u8; 8] = b.try_into().unwrap();
                let val = f64_from_bytes(&arr, endian);
                // Format as a plain number; drop trailing ".0" for whole
                // numbers so the grid reads naturally.
                if val == val.floor() && !val.is_infinite() {
                    row.push(format!("{}", val as i64));
                } else {
                    row.push(format!("{}", val));
                }
            }
        }
        cases.push(row);
    }
    Ok(cases)
}

/// Detect the endianness of an SPSS .sav file from its first bytes.
///
/// In an uncompressed SPSS .sav file, the first 4 bytes contain the bias
/// value (100). We interpret those bytes as both big- and little-endian i32
/// and pick whichever yields 100. If neither matches, we return an error
/// so the caller can try alternative strategies.
fn detect_endian(bytes: &[u8]) -> Result<Endian, String> {
    if bytes.len() < 4 {
        return Err("file too short to detect endianness".to_string());
    }
    let be = i32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    let le = i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    match (be == 100, le == 100) {
        (true, false) => Ok(Endian::Big),
        (false, true) => Ok(Endian::Little),
        (true, true) => Ok(Endian::Little), // ambiguous, default to little
        _ => Err("could not determine SAV file endianness from bias field".to_string()),
    }
}

/// Try both endiannesses and return the one that produces a valid SPSS header.
///
/// Some .sav files may not have bias=100 at byte 0, or may use a layout where
/// the first 4 bytes are ambiguous. We validate the full header (bias, n_vars,
/// name_len) for each endianness and pick the one that makes sense.
fn detect_endian_from_header(bytes: &[u8]) -> Result<Endian, String> {
    if bytes.len() < 16 {
        return Err("file too short to detect endianness".to_string());
    }

    for endian in [Endian::Little, Endian::Big] {
        let mut r = Reader::new(bytes, endian);
        if let Ok(header) = read_header(&mut r) {
            if header.bias == 100 && header.n_vars > 0 && header.name_len == 8 {
                return Ok(endian);
            }
        }
    }

    Err("could not determine SAV file endianness from header".to_string())
}

/// Parse an SPSS .sav byte buffer into a DatasetDto.
///
/// Currently supports the common uncompressed layout (bias == 100) and
/// big/little-endian files. Compressed (bias != 100) SAV data returns a
/// clear error. This is a best-effort reader: value-label and missing-value
/// binary sub-records are not decoded (they default to empty); the UI layer
/// fully supports editing them once loaded.
pub fn parse_sav(bytes: &[u8]) -> Result<DatasetDto, String> {
    if bytes.len() < 40 {
        return Err("file too short to be a valid SPSS .sav file".to_string());
    }

    // Try the fast bias-based detection first, then fall back to full-header
    // validation for files where the first 4 bytes are ambiguous.
    let endian = detect_endian(bytes)
        .or_else(|_| detect_endian_from_header(bytes))
        .unwrap_or(Endian::Little);
    let mut r = Reader::new(bytes, endian);

    let header = read_header(&mut r)?;

    let names = read_variable_names(&mut r, header.n_vars)?;
    let labels = read_variable_labels(&mut r, header.n_vars, header.label_len)?;

    // Value-label and missing-value blocks: skip (best-effort).
    let _value_labels_raw = skip_block(&mut r, header.n_vars, header.vallabel_len)?;
    let _missing_raw = skip_block(&mut r, header.n_vars, header.missing_len)?;

    let formats = read_formats(&mut r, header.n_vars, header.format_len)?;
    let measures = read_measures(&mut r, header.n_vars, header.measure_len)?;

    let mut variables = Vec::with_capacity(header.n_vars as usize);
    for i in 0..header.n_vars as usize {
        let (var_type, string_width) = formats[i].clone();
        variables.push(VariableDto {
            name: names[i].clone(),
            label: labels.get(i).cloned().unwrap_or_default(),
            var_type: var_type.clone(),
            string_width,
            measure: measures.get(i).cloned().unwrap_or_else(|| "nominal".to_string()),
            missing: None,
            value_labels: Vec::new(),
            column_width: if var_type == "string" {
                string_width.max(8)
            } else {
                12
            },
            align: if var_type == "string" { "left" } else { "right" }.to_string(),
        });
    }

    // Data (uncompressed). Reader endianness is already set from
    // `detect_endian`, which is what the file uses.
    let cases = read_data_uncompressed(&mut r, &variables, header.n_cases)?;

    Ok(DatasetDto {
        variables,
        cases,
        source_file: None,
    })
}

/// Parse an SPSS .zsav byte buffer. A .zsav file is a zlib/deflate-compressed
/// .sav file; decompress then delegate to [`parse_sav`].
pub fn parse_zsav(bytes: &[u8]) -> Result<DatasetDto, String> {
    // Try zlib-wrapped deflate first (the .zsav convention).
    let decompressed = flate2::read::ZlibDecoder::new(bytes)
        .bytes()
        .collect::<Result<Vec<u8>, _>>()
        .map_err(|e| format!("failed to decompress .zsav (zlib): {}", e))?;
    let decompressed = if decompressed.is_empty() && !bytes.is_empty() {
        // Fall back to raw deflate.
        flate2::read::DeflateDecoder::new(bytes)
            .bytes()
            .collect::<Result<Vec<u8>, _>>()
            .map_err(|e| format!("failed to decompress .zsav (deflate): {}", e))?
    } else {
        decompressed
    };
    parse_sav(&decompressed)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a minimal valid uncompressed .sav byte buffer per the layout in
    /// this module, with the given endianness, then verify round-trip.
    fn build_minimal_sav(endian: Endian) -> Vec<u8> {
        let n_vars: i32 = 2;
        let n_cases: i32 = 2;
        let bias: i32 = 100;
        let name_len: i32 = 8;
        let label_len: i32 = 16;
        let vallabel_len: i32 = 0;
        let missing_len: i32 = 0;
        let format_len: i32 = 8;
        let measure_len: i32 = 1;

        let mut buf: Vec<u8> = Vec::new();
        let w32 = |v: i32| match endian {
            Endian::Big => v.to_be_bytes(),
            Endian::Little => v.to_le_bytes(),
        };
        let w64 = |v: f64| match endian {
            Endian::Big => v.to_be_bytes(),
            Endian::Little => v.to_le_bytes(),
        };

        buf.extend_from_slice(&w32(bias)); // bias = 100 (uncompressed)
        buf.extend_from_slice(&w32(n_vars));
        buf.extend_from_slice(&w32(n_cases));
        buf.extend_from_slice(&w32(name_len));
        buf.extend_from_slice(&w32(label_len));
        buf.extend_from_slice(&w32(vallabel_len));
        buf.extend_from_slice(&w32(missing_len));
        buf.extend_from_slice(&w32(format_len));
        buf.extend_from_slice(&w32(measure_len));

        // variable names (8 bytes each)
        buf.extend_from_slice(b"AGE     ");
        buf.extend_from_slice(b"GENDER  ");

        // variable labels (16 bytes each)
        buf.extend_from_slice(b"Age in years    ");
        buf.extend_from_slice(b"Gender          ");

        // value labels (none)
        // missing values (none)

        // variable formats (8 bytes each): AGE numeric, GENDER string width 1
        buf.extend_from_slice(b"F8.2\0\0\0\0");
        buf.extend_from_slice(b"A1\0\0\0\0\0\0");

        // measure levels (1 byte each): nominal, nominal
        buf.extend_from_slice(&[1u8, 1u8]);

        // data: record_size = 8 (AGE) + 1 (GENDER) = 9 bytes per case
        // case 1: AGE=25.0, GENDER="M"
        buf.extend_from_slice(&w64(25.0));
        buf.push(b'M');
        // case 2: AGE=30.0, GENDER="F"
        buf.extend_from_slice(&w64(30.0));
        buf.push(b'F');

        buf
    }

    #[test]
    fn round_trip_big_endian() {
        let bytes = build_minimal_sav(Endian::Big);
        let ds = parse_sav(&bytes).expect("big-endian sav should parse");
        assert_eq!(ds.variables.len(), 2);
        assert_eq!(ds.variables[0].name, "AGE");
        assert_eq!(ds.variables[0].var_type, "numeric");
        assert_eq!(ds.variables[1].name, "GENDER");
        assert_eq!(ds.variables[1].var_type, "string");
        assert_eq!(ds.variables[1].string_width, 1);
        assert_eq!(ds.cases.len(), 2);
        assert_eq!(ds.cases[0], vec!["25".to_string(), "M".to_string()]);
        assert_eq!(ds.cases[1], vec!["30".to_string(), "F".to_string()]);
        assert_eq!(ds.variables[0].label, "Age in years");
    }

    #[test]
    fn round_trip_little_endian() {
        let bytes = build_minimal_sav(Endian::Little);
        let ds = parse_sav(&bytes).expect("little-endian sav should parse");
        assert_eq!(ds.variables.len(), 2);
        assert_eq!(ds.cases.len(), 2);
        assert_eq!(ds.cases[0], vec!["25".to_string(), "M".to_string()]);
    }

    #[test]
    fn compressed_sav_rejected() {
        let mut bytes = build_minimal_sav(Endian::Little);
        // Flip the bias field (offset 0) from 100 to 200 to signal compressed.
        let b: [u8; 4] = 200i32.to_le_bytes();
        bytes[0..4].copy_from_slice(&b);
        let result = parse_sav(&bytes);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("compressed"));
    }

    #[test]
    fn zsav_round_trip() {
        let sav = build_minimal_sav(Endian::Little);
        let mut compressed = Vec::new();
        {
            let mut encoder = flate2::write::ZlibEncoder::new(
                &mut compressed,
                flate2::Compression::default(),
            );
            use std::io::Write;
            encoder.write_all(&sav).unwrap();
            encoder.finish().unwrap();
        }
        let ds = parse_zsav(&compressed).expect("zsav should parse");
        assert_eq!(ds.variables.len(), 2);
        assert_eq!(ds.cases.len(), 2);
    }

    #[test]
    fn too_short_rejected() {
        assert!(parse_sav(&[1, 2, 3]).is_err());
    }
}



