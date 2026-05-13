use crate::engine::state::position::{Position, PositionStatus, PositionType};
use chrono::{DateTime, NaiveWeek, Utc};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, VecDeque},
    mem::Discriminant,
    thread::sleep,
};
use uuid::Uuid;

pub struct Engine {
    pub asset: String,
    pub positions: HashMap<Uuid, Position>,
    pub current_price: Decimal,
    pub price_history: VecDeque<Decimal>,
    pub funding_rate: Decimal,

    pub mark_price: Decimal,
    pub maintenance_margin_rate: Decimal,
    pub last_funding_time: DateTime<Utc>,
    pub max_leverage: Decimal,
    pub insurance_fund: Decimal,
    pub max_positions: usize,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EngineConfig {
    asset: String,
    max_leverage: Decimal,
    maintenance_margin_rate: Decimal,
    funding_rate: Decimal,
    max_positions: usize,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OpenPositionData {
    user_id: Uuid,
    asset: String,
    side: PositionType,
    quantity: Decimal,
    leverage: Decimal,
    margin: Decimal,
}
impl Engine {
    pub fn new(config: EngineConfig) -> Self {
        Self {
            asset: config.asset,
            positions: HashMap::new(),
            last_funding_time: Utc::now(),
            price_history: VecDeque::new(),
            mark_price: dec!(0),
            current_price: dec!(0),
            insurance_fund: dec!(0),
            maintenance_margin_rate: config.maintenance_margin_rate,
            funding_rate: config.funding_rate,
            max_leverage: config.max_leverage,
            max_positions: config.max_positions,
        }
    }

    pub fn open_position(&mut self, open_position_data: OpenPositionData) -> Result<Uuid, String> {
        self.validate_open_position(&open_position_data)?;

        let id = Uuid::new_v4();
        let current_price = self.current_price;
        let margin = open_position_data.margin;
        let position_size = open_position_data.leverage * margin;
        let quantity = position_size / current_price;
        let maintainance_margin = position_size * self.maintenance_margin_rate;

        let liquidation_price = match open_position_data.side {
            PositionType::Long => current_price - (margin - maintainance_margin) / quantity,
            PositionType::Short => current_price + (margin - maintainance_margin) / quantity,
        };

        let now = Utc::now();

        let position = Position {
            user_id: open_position_data.user_id,
            side: open_position_data.side,
            asset: open_position_data.asset,
            entry_price: current_price,
            leverage: open_position_data.leverage,
            margin: open_position_data.margin,
            quantity,
            liquidation_price,
            pnl: dec!(0),
            opened_at: now,
            status: PositionStatus::Open,
        };

        self.positions.entry(id).insert_entry(position);

        Ok(id)
    }

    pub fn update_price(&mut self, new_price: Decimal) -> Result<(), String> {
        self.current_price = new_price;
        self.price_history.push_back(new_price);
        if self.price_history.len() > 10 {
            self.price_history.pop_front();
        }
        if self.mark_price.is_zero() {
            self.mark_price = new_price
        } else {
            let sum: Decimal = self.price_history.iter().sum();
            self.mark_price = sum / Decimal::from(self.price_history.len());
        }

        for position in self.positions.values_mut() {
            let pnl: Decimal;
            match position.side {
                PositionType::Long => {
                    pnl = (self.mark_price - position.entry_price) * position.quantity;
                }
                PositionType::Short => {
                    pnl = (position.entry_price - self.mark_price) * position.quantity;
                }
            }
            position.pnl = pnl;
        }

        Ok(())
    }
    pub fn validate_open_position(&self, data: &OpenPositionData) -> Result<(), String> {
        if data.asset != self.asset {
            return Err("Invalid asset".to_string());
        }
        if data.leverage > self.max_leverage {
            return Err(format!("Max leverage is {}", self.max_leverage));
        }
        if data.leverage <= Decimal::ZERO {
            return Err("Leverage must be > 0".to_string());
        }
        if data.margin <= Decimal::ZERO {
            return Err("Margin must be > 0".to_string());
        }
        if self.positions.len() >= self.max_positions {
            return Err("Market at max capacity".to_string());
        }
        Ok(())
    }
}
