use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum CloseReason {
    Manual,
    Liquidated,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Trade {
    pub id: Uuid,
    pub position_id: Uuid,
    pub user_id: Uuid,

    pub asset: String,
    pub close_reason: CloseReason,

    pub entry_price: Decimal,
    pub exit_price: Decimal,

    pub settled_at: DateTime<Utc>,
}
