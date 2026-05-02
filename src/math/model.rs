// ========== DOSYA: sentinel-core/src/math/model.rs ==========
use crate::types::SignalType;
use ndarray::{Array1, Array2};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ModelError {
    #[error("Matrix shape mismatch: {0}")]
    ShapeMismatch(#[from] ndarray::ShapeError),
    #[error("Empty logits array")]
    EmptyLogits,
}

pub struct PureMathModel {
    weights: Array2<f32>,
    biases: Array1<f32>,
}

impl PureMathModel {
    pub fn new(weights_data: Vec<f32>, biases_data: Vec<f32>) -> Result<Self, ModelError> {
        let weights = Array2::from_shape_vec((12, 3), weights_data)?;
        let biases = Array1::from_vec(biases_data);
        Ok(Self { weights, biases })
    }

    pub fn predict(&self, features: &[f32; 12]) -> Result<(SignalType, f64), ModelError> {
        let input = Array1::from_vec(features.to_vec());
        let logits = input.dot(&self.weights) + &self.biases;

        // Softmax
        let max_logit = logits.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        let exp_logits = logits.mapv(|x| (x - max_logit).exp());
        let sum_exp = exp_logits.sum();

        if sum_exp == 0.0 {
            return Err(ModelError::EmptyLogits);
        }

        let probs = exp_logits / sum_exp;

        let hold_prob = probs[0];
        let buy_prob = probs[1];
        let sell_prob = probs[2];

        if buy_prob > hold_prob && buy_prob > sell_prob {
            Ok((SignalType::Buy, buy_prob as f64))
        } else if sell_prob > hold_prob && sell_prob > buy_prob {
            Ok((SignalType::Sell, sell_prob as f64))
        } else {
            Ok((SignalType::Hold, hold_prob as f64))
        }
    }
}
