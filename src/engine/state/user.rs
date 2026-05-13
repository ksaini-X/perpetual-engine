use rust_decimal::Decimal;
use uuid::Uuid;
pub struct User {
    pub id: Uuid,
    pub balance: Decimal,
    pub available_balance: Decimal,
    pub locked_margin: Decimal,
}
