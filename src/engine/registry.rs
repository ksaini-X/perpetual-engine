use crate::engine::engine::Engine;
use crate::engine::state::{trade::Trade, user::User};
use std::collections::HashMap;

pub struct Registry {
    pub enignes: HashMap<String, Engine>,
    pub trades: HashMap<String, Trade>,
    pub users: HashMap<String, User>,
}
