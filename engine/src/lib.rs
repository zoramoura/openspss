use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json;

mod sav;
use sav::{parse_sav, parse_zsav};

#[derive(Serialize, Deserialize)]
pub struct DescriptivesResult {
    pub count: usize,
    pub mean: f64,
    pub variance: f64,
    pub std_dev: f64,
    pub min: f64,
    pub max: f64,
}

#[wasm_bindgen]
pub fn compute_descriptives(data: &[f64]) -> Result<JsValue, JsValue> {
    if data.is_empty() {
        return Err(JsValue::from_str("Data is empty"));
    }

    let count = data.len();
    let sum: f64 = data.iter().sum();
    let mean = sum / count as f64;

    let squared_diffs: Vec<f64> = data.iter().map(|&x| (x - mean).powi(2)).collect();
    let variance = squared_diffs.iter().sum::<f64>() / count as f64;
    let std_dev = variance.sqrt();

    let min = *data.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
    let max = *data.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();

    let result = DescriptivesResult {
        count,
        mean,
        variance,
        std_dev,
        min,
        max,
    };

    serde_wasm_bindgen::to_value(&result).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Read an SPSS .sav file. Returns a JSON-serialised [`DatasetDto`] string.
#[wasm_bindgen]
pub fn read_sav(bytes: &[u8]) -> Result<String, JsValue> {
    parse_sav(bytes)
        .map(|ds| serde_json::to_string(&ds))
        .map_err(|e| JsValue::from_str(&e))?
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Read an SPSS .zsav (deflate-compressed .sav) file. Returns a JSON-serialised
/// [`DatasetDto`] string.
#[wasm_bindgen]
pub fn read_zsav(bytes: &[u8]) -> Result<String, JsValue> {
    parse_zsav(bytes)
        .and_then(|ds| serde_json::to_string(&ds).map_err(|e| e.to_string()))
        .map_err(|e| JsValue::from_str(&e))
}
