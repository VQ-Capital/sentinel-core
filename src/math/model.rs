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
    w1: Array2<f32>,
    b1: Array1<f32>,
    w2: Array2<f32>,
    b2: Array1<f32>,
}

impl PureMathModel {
    pub fn new(
        w1_data: Vec<f32>,
        b1_data: Vec<f32>,
        w2_data: Vec<f32>,
        b2_data: Vec<f32>,
    ) -> Result<Self, ModelError> {
        let w1 = Array2::from_shape_vec((12, 8), w1_data)?;
        let b1 = Array1::from_vec(b1_data);
        let w2 = Array2::from_shape_vec((8, 3), w2_data)?;
        let b2 = Array1::from_vec(b2_data);
        Ok(Self { w1, b1, w2, b2 })
    }

    pub fn predict(&self, features: &[f32; 12]) -> Result<(SignalType, f64), ModelError> {
        let input = Array1::from_vec(features.to_vec());

        // 🔥 CERRAHİ: Hidden Layer (X * W1 + B1) -> Non-Linear ReLU Aktivasyonu
        let mut hidden = input.dot(&self.w1) + &self.b1;
        hidden.mapv_inplace(|x| x.max(0.0));

        // Output Layer (Hidden * W2 + B2)
        let logits = hidden.dot(&self.w2) + &self.b2;

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
