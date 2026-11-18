use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};

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