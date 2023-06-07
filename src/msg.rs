
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use crate::types::Bet;

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub min_bet: Option<u64>,
    pub max_bet: Option<u64>,
    pub max_total: Option<u64>,
    pub supported_denoms: Option<Vec<String>>
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Bet {
        bets: Vec<Bet>
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // User that wants to read their share (todo: authentication)
    // ReadShare {
    //     user_index: u32
    // }
}
