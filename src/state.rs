use secret_toolkit_storage::Item;
use cosmwasm_std::{StdResult, Storage, Addr};
use serde::{Deserialize, Serialize};

pub static CONFIG_KEY: &str = "config";
pub static ADMIN_KEY: &str = "admin";

pub static CONFIG_ITEM: Item<Config> = Item::new(CONFIG_KEY.as_bytes());
pub static ADMIN_ITEM: Item<Addr> = Item::new(ADMIN_KEY.as_bytes());

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    pub min_bet: u64,
    pub max_bet: u64,
    pub max_total: u64,
    pub supported_denoms: Vec<String>
}

pub fn save_config(storage: &mut dyn Storage, config: &Config) -> StdResult<()> {
    CONFIG_ITEM.save(storage, config)
}

pub fn load_config(storage: &dyn Storage) -> StdResult<Config> {
    CONFIG_ITEM.load(storage)
}

pub fn save_admin(storage: &mut dyn Storage, admin: &Addr) -> StdResult<()> {
    ADMIN_ITEM.save(storage, admin)
}

pub fn load_admin(storage: &dyn Storage) -> StdResult<Addr> {
    ADMIN_ITEM.load(storage)
}