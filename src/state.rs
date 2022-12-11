use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

use cosmwasm_std::{Addr, Coin, StdResult, Storage};
use cw_storage_plus::Map;

pub static CONFIG_KEY: &[u8] = b"config";

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RPS {
    Rock,
    Paper,
    Scissors,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, JsonSchema)]
pub struct RPSMatch {
    pub meta: GameMetaInfo,
    pub status: GameStatus,
    pub players: [Player; 2],
    pub bet: Option<Coin>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, Default, JsonSchema)]
pub struct GameMetaInfo {
    pub end_game_block: Option<u64>,
    pub winner: Option<GameResult>,
}

#[derive(Serialize, Deserialize, Copy, PartialEq, Clone, Debug, JsonSchema)]
pub enum GameStatus {
    Initialized = 0,
    WaitingForPlayerToJoin,
    Started,
    Got1stChoiceWaitingFor2nd,
    Done,
    WaitingForWinner,
}

impl RPSMatch {
    pub fn next(&mut self) {
        match self.status {
            GameStatus::Initialized => self.status = GameStatus::WaitingForPlayerToJoin,
            GameStatus::WaitingForPlayerToJoin => self.status = GameStatus::Started,
            GameStatus::Started => self.status = GameStatus::Got1stChoiceWaitingFor2nd,
            GameStatus::Got1stChoiceWaitingFor2nd => self.status = GameStatus::Done,
            GameStatus::Done => self.status = GameStatus::Initialized,
            _ => {}
        }
    }
}

impl Default for GameStatus {
    fn default() -> Self {
        Self::Initialized
    }
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Eq, Debug, JsonSchema)]
pub enum GameResult {
    Player1,
    Player2,
    Tie,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, JsonSchema)]
pub struct Player {
    name: String,
    address: Addr,
    pub choice: Option<RPS>,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            name: "".to_string(),
            address: Addr::unchecked("".to_string()),
            choice: None,
        }
    }
}

impl Player {
    pub fn new(name: String, address: Addr) -> Player {
        return Player {
            name,
            address,
            choice: None,
        };
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn address(&self) -> &Addr {
        &self.address
    }
}

impl PartialEq for Player {
    fn eq(&self, other: &Self) -> bool {
        self.address == other.address
    }
}

pub fn save_match_info(storage: &mut dyn Storage, game: &str, state: RPSMatch) -> StdResult<()> {
    const GAME_STATE: Map<&[u8], RPSMatch> = Map::new("game_state");
    GAME_STATE.save(storage, game.as_bytes(), &state)
}

pub fn load_match_info(storage: &dyn Storage, game: &str) -> StdResult<RPSMatch> {
    const GAME_STATE: Map<&[u8], RPSMatch> = Map::new("game_state");
    GAME_STATE.load(storage, game.as_bytes())
}

pub fn calculate_winner(p1: &RPS, p2: &RPS) -> GameResult {
    match (p1, p2) {
        (RPS::Rock, RPS::Rock) => GameResult::Tie,
        (RPS::Rock, RPS::Paper) => GameResult::Player2,
        (RPS::Rock, RPS::Scissors) => GameResult::Player1,
        (RPS::Paper, RPS::Paper) => GameResult::Tie,
        (RPS::Paper, RPS::Rock) => GameResult::Player1,
        (RPS::Paper, RPS::Scissors) => GameResult::Player2,
        (RPS::Scissors, RPS::Scissors) => GameResult::Tie,
        (RPS::Scissors, RPS::Rock) => GameResult::Player2,
        (RPS::Scissors, RPS::Paper) => GameResult::Player1,
    }
}
