use cosmwasm_std::{Addr, Coin, Uint128};
use serde::{Deserialize, Serialize};
use crate::state::{GameResult, RPS};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct InstantiateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    NewGame { bet: Option<Coin>, name: String },
    JoinGame { game: String },
    SubmitChoice { game: String, choice: RPS },
    Finalize { game: String }
    // Reset {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    WhoWon { game: String },
}

/// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CheckWinner {
    pub winner: GameResult,
    pub address: Option<Addr>
}