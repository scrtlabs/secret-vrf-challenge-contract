use crate::state::{GameResult, RPS};
use cosmwasm_std::{Addr, Coin};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct InstantiateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    NewGame { bet: Option<Coin>, name: String },
    JoinGame { name: String, game: String },
    SubmitChoice { game: String, choice: RPS },
    Finalize { game: String }, // Reset {},
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
    pub address: Option<Addr>,
}
