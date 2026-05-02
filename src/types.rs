// ========== DOSYA: sentinel-core/src/types.rs ==========

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignalType {
    Unspecified = 0,
    Hold = 1,
    StrongBuy = 2,
    Buy = 3,
    Sell = 4,
    StrongSell = 5,
}

#[derive(Debug, Clone)]
pub struct TradeSignal {
    pub symbol: String,
    pub signal_type: SignalType,
    pub confidence_score: f64,
    pub recommended_leverage: f64,
    pub timestamp: i64,
}

#[derive(Clone, Default, Debug)]
pub struct Position {
    pub quantity: f64,
    pub avg_price: f64,
    pub entry_time: i64,
}

#[derive(Clone, Copy)]
pub struct SymbolRules {
    pub tick_size: f64,
    pub step_size: f64,
    pub min_notional: f64,
}

// Borsa kuralları (Mock/Static)
pub fn get_symbol_rules(symbol: &str) -> SymbolRules {
    match symbol {
        "BTCUSDT" => SymbolRules {
            tick_size: 0.1,
            step_size: 0.00001,
            min_notional: 5.0,
        },
        "ETHUSDT" => SymbolRules {
            tick_size: 0.01,
            step_size: 0.0001,
            min_notional: 5.0,
        },
        "SOLUSDT" => SymbolRules {
            tick_size: 0.01,
            step_size: 0.01,
            min_notional: 5.0,
        },
        _ => SymbolRules {
            tick_size: 0.001,
            step_size: 0.1,
            min_notional: 5.0,
        },
    }
}

pub fn format_precision(val: f64, step: f64) -> f64 {
    let inv = 1.0 / step;
    (val * inv).trunc() / inv
}
