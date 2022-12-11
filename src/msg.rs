use crate::state::{GameResult, RPS};
use cosmwasm_std::{Addr, Coin};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, JsonSchema)]
pub struct InstantiateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    NewGame {
        player_name: String,
        bet: Option<Coin>,
    },
    JoinGame {
        player_name: String,
        game_code: String,
    },
    SubmitChoice {
        game_code: String,
        choice: RPS,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    WhoWon { game: String },
    GameState { game: String },
}

/// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CheckWinner {
    pub winner: GameResult,
    pub address: Option<Addr>,
}

/// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GameStateResponse {
    pub game: String,
    pub state: crate::state::GameStatus,
}
