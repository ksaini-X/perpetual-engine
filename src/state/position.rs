use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum PositionType {
    Long,
    Short,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum PositionStatus {
    Open,
    Closed,     // user closed manually
    Liquidated, // force closed
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Position {
    pub id: Uuid,
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
    pub closed_at: DateTime<Utc>,
}
