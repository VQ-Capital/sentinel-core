// ========== DOSYA: sentinel-core/src/risk/engine.rs ==========
use crate::types::{format_precision, get_symbol_rules, Position, SignalType, TradeSignal};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct RiskConfig {
    pub initial_balance: f64,
    pub max_drawdown_usd: f64,
    pub defensive_drawdown_usd: f64,
    pub cooldown_ms: i64,
    pub min_hold_time_ms: i64,
    pub max_hold_time_ms: i64,
    pub base_risk_pct: f64,
    pub base_leverage: f64,
    pub take_profit_pct: f64,
    pub stop_loss_pct: f64,
}

pub struct RiskEngine {
    config: RiskConfig,
    pub positions: HashMap<String, Position>,
    pub last_trade_time: HashMap<String, i64>,
    pub kill_switch_active: bool,
    pub is_defensive_mode: bool,
}

impl RiskEngine {
    pub fn new(config: RiskConfig) -> Self {
        Self {
            config,
            positions: HashMap::new(),
            last_trade_time: HashMap::new(),
            kill_switch_active: false,
            is_defensive_mode: false,
        }
    }

    pub fn auto_tune_risk(&mut self, current_equity: f64) {
        if self.kill_switch_active {
            return;
        }
        let drawdown_usd = self.config.initial_balance - current_equity;

        if drawdown_usd > self.config.defensive_drawdown_usd && !self.is_defensive_mode {
            self.is_defensive_mode = true;
        }

        if drawdown_usd >= self.config.max_drawdown_usd && !self.kill_switch_active {
            self.kill_switch_active = true;
        }
    }

    pub fn evaluate_signal(
        &mut self,
        signal: &TradeSignal,
        price: f64,
        equity: f64,
        current_time_ms: i64,
    ) -> Result<f64, &'static str> {
        if self.kill_switch_active {
            return Err("KILL_SWITCH_ENGAGED");
        }

        let side = match signal.signal_type {
            SignalType::Buy | SignalType::StrongBuy => "BUY",
            SignalType::Sell | SignalType::StrongSell => "SELL",
            _ => return Err("INVALID_SIDE"),
        };

        if let Some(pos) = self.positions.get(&signal.symbol) {
            if pos.quantity.abs() > 1e-6
                && ((side == "BUY" && pos.quantity > 0.0) || (side == "SELL" && pos.quantity < 0.0))
            {
                return Err("ANTI_MARTINGALE_REJECT");
            }
        }

        let last_time = self
            .last_trade_time
            .get(&signal.symbol)
            .copied()
            .unwrap_or(0);
        if current_time_ms - last_time < self.config.cooldown_ms {
            return Err("COOLDOWN_ACTIVE");
        }

        let signal_strength = match signal.signal_type {
            SignalType::StrongBuy | SignalType::StrongSell => 1.0,
            _ => 0.5,
        };

        let active_risk = if self.is_defensive_mode {
            self.config.base_risk_pct * 0.5
        } else {
            self.config.base_risk_pct
        };

        let active_leverage = if self.is_defensive_mode {
            self.config.base_leverage * 0.5
        } else {
            self.config.base_leverage
        };

        let raw_quantity = (equity * active_risk * signal_strength * active_leverage) / price;
        let rules = get_symbol_rules(&signal.symbol);

        let notional_value = raw_quantity * price;
        if notional_value < rules.min_notional {
            return Err("MIN_NOTIONAL_REJECTED");
        }

        let formatted_qty = format_precision(raw_quantity, rules.step_size);
        if formatted_qty <= 0.0 {
            return Err("INSUFFICIENT_MARGIN");
        }

        self.last_trade_time
            .insert(signal.symbol.clone(), current_time_ms);
        Ok(formatted_qty)
    }

    pub fn check_tp_sl(
        &mut self,
        current_prices: &HashMap<String, f64>,
        current_time_ms: i64,
    ) -> Vec<(String, &'static str, f64, f64)> {
        let mut orders = Vec::new();

        for (symbol, pos) in self.positions.iter() {
            if pos.quantity.abs() < 1e-6 {
                continue;
            }
            if let Some(&price) = current_prices.get(symbol) {
                if self.kill_switch_active {
                    orders.push((
                        symbol.clone(),
                        if pos.quantity > 0.0 { "SELL" } else { "BUY" },
                        pos.quantity.abs(),
                        price,
                    ));
                    continue;
                }

                let pnl = if pos.quantity > 0.0 {
                    (price - pos.avg_price) / pos.avg_price
                } else {
                    (pos.avg_price - price) / pos.avg_price
                };
                let time_held = current_time_ms - pos.entry_time;

                if time_held < self.config.min_hold_time_ms && pnl > -self.config.stop_loss_pct {
                    continue;
                }

                if pnl >= self.config.take_profit_pct
                    || pnl <= -self.config.stop_loss_pct
                    || (time_held > self.config.max_hold_time_ms)
                {
                    orders.push((
                        symbol.clone(),
                        if pos.quantity > 0.0 { "SELL" } else { "BUY" },
                        pos.quantity.abs(),
                        price,
                    ));
                }
            }
        }
        orders
    }

    // İşlem gerçekleştiğinde portföyü güncelle
    pub fn process_execution(
        &mut self,
        symbol: &str,
        side: &str,
        exec_price: f64,
        qty: f64,
        timestamp: i64,
    ) -> f64 {
        let pos = self.positions.entry(symbol.to_string()).or_default();
        let mut realized = 0.0;

        if (side == "SELL" && pos.quantity > 0.0) || (side == "BUY" && pos.quantity < 0.0) {
            let close_qty = qty.min(pos.quantity.abs());
            realized = if pos.quantity > 0.0 {
                (exec_price - pos.avg_price) * close_qty
            } else {
                (pos.avg_price - exec_price) * close_qty
            };

            pos.quantity = if pos.quantity > 0.0 {
                pos.quantity - close_qty
            } else {
                pos.quantity + close_qty
            };

            if pos.quantity.abs() < 1e-6 {
                pos.avg_price = 0.0;
            }
        } else {
            let new_qty = if side == "BUY" {
                pos.quantity + qty
            } else {
                pos.quantity - qty
            };
            pos.avg_price =
                ((pos.quantity.abs() * pos.avg_price) + (qty * exec_price)) / new_qty.abs();
            pos.quantity = new_qty;
            pos.entry_time = timestamp;
        }
        realized
    }
}
