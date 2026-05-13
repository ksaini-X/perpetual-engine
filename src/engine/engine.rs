use crate::engine::{
    registry::EngineConfig,
    state::{
        position::{OpenPositionData, Position, PositionStatus, PositionType},
        trade::{CloseReason, Trade},
    },
};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
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
    pub funds: Decimal,
    pub max_positions: usize,
    pub trades: Vec<Trade>,
}

impl Engine {
    pub fn new(config: EngineConfig) -> Self {
        Self {
            asset: config.asset,
            positions: HashMap::new(),
            trades: Vec::new(),
            last_funding_time: Utc::now(),
            price_history: VecDeque::new(),
            mark_price: dec!(0),
            current_price: dec!(0),
            funds: dec!(0),
            maintenance_margin_rate: config.maintenance_margin_rate,
            funding_rate: dec!(0),
            max_leverage: config.max_leverage,
            max_positions: config.max_positions,
        }
    }

    pub fn open_position(&mut self, open_position_data: OpenPositionData) -> Result<Uuid, String> {
        self.validate_open_position_data(&open_position_data)?;

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

        let positions_to_liquate: Vec<Uuid> = self
            .positions
            .iter()
            .filter(|(_id, position)| {
                position.pnl + position.margin <= position.margin * self.maintenance_margin_rate
            })
            .map(|(id, _position)| *id)
            .collect();

        for id in positions_to_liquate {
            self.liqudate_position(id).unwrap()
        }

        self.funding_rate = (self.current_price - self.mark_price) / self.mark_price;

        if Utc::now().timestamp() - self.last_funding_time.timestamp() >= 8 * 60 * 60 {
            self.apply_funding();
        }

        Ok(())
    }

    fn liqudate_position(&mut self, id: Uuid) -> Result<(), String> {
        let position = self.positions.remove(&id).expect("Failed to liquate");

        self.funds += (position.margin + position.pnl).max(dec!(0));

        let trade = Trade {
            asset: self.asset.clone(),
            close_reason: CloseReason::Liquidated,
            entry_price: position.entry_price,
            exit_price: self.mark_price,
            id: Uuid::new_v4(),
            position_id: id,
            settled_at: Utc::now(),
            user_id: position.user_id,
        };

        self.trades.push(trade);
        Ok(())
    }

    pub fn validate_open_position_data(&self, data: &OpenPositionData) -> Result<(), String> {
        if self.positions.len() >= self.max_positions {
            return Err("Market at max capacity".to_string());
        }
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
        Ok(())
    }

    pub fn apply_funding(&mut self) {
        let mut total_fund: Decimal = dec!(0);
        let mut total_side_receipt: Decimal = dec!(0);

        if self.funding_rate > dec!(0) {
            for (_id, position) in self.positions.iter_mut() {
                if position.side == PositionType::Long {
                    let deduction = position.quantity * self.mark_price * self.funding_rate;
                    position.margin -= deduction;
                    total_fund += deduction;
                } else {
                    total_side_receipt += position.quantity * self.mark_price;
                }
            }
        } else {
            for (_id, position) in self.positions.iter_mut() {
                if position.side == PositionType::Short {
                    let deduction = position.quantity * self.mark_price * self.funding_rate;
                    position.margin -= deduction;
                    total_fund += deduction;
                } else {
                    total_side_receipt += position.quantity * self.mark_price;
                }
            }
        }

        if self.funding_rate > dec!(0) {
            for (_id, position) in self.positions.iter_mut() {
                if position.side == PositionType::Short {
                    position.margin +=
                        ((position.quantity * self.mark_price) / total_side_receipt) * total_fund;
                }
            }
        } else {
            for (_id, position) in self.positions.iter_mut() {
                if position.side == PositionType::Long {
                    position.margin +=
                        ((position.quantity * self.mark_price) / total_side_receipt) * total_fund;
                }
            }
        }
    }
}
