use cw_storage_plus::Item;
use cosmwasm_std::{StdResult, Storage};
use serde::{Deserialize, Serialize};

pub static CONFIG_KEY: &str = "config";

pub static CONFIG_ITEM: Item<Config> = Item::new(CONFIG_KEY);

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    pub min_bet: u64,
    pub max_bet: u64
}

pub fn save_config(storage: &mut dyn Storage, config: &Config) -> StdResult<()> {
    CONFIG_ITEM.save(storage, config)
}

pub fn load_config(storage: &dyn Storage) -> StdResult<Config> {
    CONFIG_ITEM.load(storage)
}
