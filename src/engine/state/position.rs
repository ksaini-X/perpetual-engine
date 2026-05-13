use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum PositionStatus {
    Open,
    Closed,
    Liquidated,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Position {
    pub user_id: Uuid,
    pub asset: String,
    pub side: PositionType,
    pub status: PositionStatus,

    pub entry_price: Decimal,

    pub quantity: Decimal,
    pub leverage: Decimal,

    pub margin: Decimal,
    pub liquidation_price: Decimal,

    pub pnl: Decimal,

    pub opened_at: DateTime<Utc>,
}
#[derive(Debug, Serialize, Deserialize, Clone)]

pub struct OpenPositionData {
    pub user_id: Uuid,
    pub asset: String,
    pub side: PositionType,
    pub quantity: Decimal,
    pub leverage: Decimal,
    pub margin: Decimal,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum PositionType {
    Long,
    Short,
}
