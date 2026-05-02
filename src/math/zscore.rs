// ========== DOSYA: sentinel-core/src/math/zscore.rs ==========
#[derive(Clone, Debug)]
pub struct OnlineZScore {
    mean: f64,
    variance: f64,
    alpha: f64,
    initialized: bool,
}

impl OnlineZScore {
    pub fn new(window: usize) -> Self {
        Self {
            mean: 0.0,
            variance: 1.0,
            alpha: 2.0 / (window as f64 + 1.0),
            initialized: false,
        }
    }

    pub fn update(&mut self, mut value: f64, scale: f64) -> f64 {
        value *= scale;
        if !self.initialized {
            self.mean = value;
            self.variance = 1.0;
            self.initialized = true;
            return 0.0;
        }
        let diff = value - self.mean;
        self.mean += self.alpha * diff;
        self.variance = (1.0 - self.alpha) * (self.variance + self.alpha * diff * diff);

        let std_dev = self.variance.sqrt();
        if std_dev < 1e-6 {
            0.0
        } else {
            ((value - self.mean) / std_dev).clamp(-3.0, 3.0)
        }
    }
}
