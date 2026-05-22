use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::engine::engine::Engine;
use crate::engine::state::position::OpenPositionData;
use crate::engine::state::{trade::Trade, user::User};
use std::collections::HashMap;

pub struct Registry {
    pub enignes: HashMap<String, Engine>,
    //TODO : move trades from engine to Registry
    pub trades: HashMap<Uuid, Vec<Trade>>,
    pub users: HashMap<Uuid, User>,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EngineConfig {
    pub asset: String,
    pub max_leverage: Decimal,
    pub maintenance_margin_rate: Decimal,
    pub max_positions: usize,
}
impl Registry {
    pub fn new() -> Self {
        Self {
            enignes: HashMap::new(),
            trades: HashMap::new(),
            users: HashMap::new(),
        }
    }
    pub fn new_engine(&mut self, engine_config: EngineConfig) -> Result<(), String> {
        if self.enignes.contains_key(&engine_config.asset) {
            Err("Engine for this asset Exists".to_string())
        } else {
            self.enignes
                .insert(engine_config.asset.clone(), Engine::new(engine_config));
            Ok(())
        }
    }

    pub fn price_update(&mut self, asset: String, new_price: Decimal) -> Result<(), String> {
        let engine = self.enignes.get_mut(&asset).ok_or("Engine not found")?;
        engine.update_price(new_price)
    }

    pub fn open_position(&mut self, open_position_data: OpenPositionData) -> Result<Uuid, String> {
        self.validate_user_balance(open_position_data.margin, &open_position_data.user_id)?;
        let engine = self.enignes.get_mut(&open_position_data.asset);
        match engine {
            Some(engine) => engine.open_position(open_position_data),
            None => Err("Asset engine not found".to_string()),
        }
    }

    fn validate_user_balance(&mut self, margin: Decimal, user_id: &Uuid) -> Result<(), String> {
        let user = self
            .users
            .get(user_id)
            .ok_or("User not found".to_string())?;
        if user.balance <= margin {
            return Err("Insufficeint balance".to_string());
        }
        let user = self.users.get_mut(&user_id).unwrap();
        user.balance -= margin;
        user.locked_margin += margin;
        user.available_balance -= margin;
        Ok(())
    }

    pub fn add_user(&mut self) -> Result<&User, String> {
        let id = Uuid::new_v4();
        let user = User {
            id,
            available_balance: dec!(10_000_000),
            balance: dec!(10_000_000),
            locked_margin: dec!(0),
        };
        self.users.insert(id, user);
        Ok(&self.users.get(&id).unwrap())
    }

    pub fn get_user(&mut self, user_id: Uuid) -> Result<&User, String> {
        self.users
            .get(&user_id)
            .ok_or("Invalid User ID".to_string())
    }
}
