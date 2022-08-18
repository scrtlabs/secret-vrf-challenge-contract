use std::cmp::Ordering;

use cosmwasm_std::{Storage, Addr, Coin};

use cosmwasm_storage::{
    ReadonlySingleton, singleton, Singleton,
    singleton_read,
};

use serde::{Deserialize, Serialize};

const CONFIG_KEY: &[u8] = b"config";

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub enum RPS {
    Rock,
    Paper,
    Scissors
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct State {
    pub game_state: GameState,
    pub players: [Player; 2],
    pub choices: [Option<RPS>; 2],
    pub bet: Option<Coin>,
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug, Default)]
pub struct GameState {
    pub status: CurrentStatus,
    pub end_game_block: Option<u64>,
    pub winner: Option<GameResult>
}

impl State {
    pub fn next(mut self) {
        // make sure to check the status before advancing it
        if &self.game_state.status == CurrentStatus::Got1stChoice {
            self.game_state.winner = Some(calculate_winner(&self.choices[0].unwrap(), &self.choices[1].unwrap()));
        }
        self.game_state.status.next();
    }
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub enum GameResult {
    Player1,
    Player2,
    Tie
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub enum CurrentStatus {
    Initialized,
    WaitingForPlayerToJoin,
    Started,
    Got1stChoice,
    DoneGettingChoices,
    Finalized
}

impl CurrentStatus {
    pub(crate) fn next(mut self) {
        match &self {
            Self::Initialized => { self = Self::WaitingForPlayerToJoin },
            Self::WaitingForPlayerToJoin => {
                self = Self::Started
            },
            Self::Started => {
                self = Self::Got1stChoice
            },
            Self::Got1stChoice => {
                self = Self::DoneGettingChoices
            },
            Self::DoneGettingChoices => {
                self = Self::Finalized
            },
            Self::Finalized => {
                self = Self::Started
            }
        }
    }

    fn reset(mut self) {
        self = Self::Init;
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self::Init
    }
}

impl From<u8> for GameState {
    fn from(num: u8) -> Self {
        match num {
            0 => GameState::Init,
            1 => GameState::GotFromPlayer1,
            2 => GameState::GotFromPlayer2,
            3 => GameState::Finalized,
            _ => GameState::Init
        }
    }
}

impl From<GameState> for u8 {
    fn from(state: GameState) -> Self {
        match state {
            GameState::Init => 0,
            GameState::GotFromPlayer1 => 1,
            GameState::GotFromPlayer2 => 2,
            GameState::Finalized => 3
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, Eq)]
pub struct Player {
    name: String,
    address: Addr,
}

impl Player {
    /// Constructor function. Takes input parameters and initializes a struct containing both
    /// those items
    pub fn new(name: String, address: Addr) -> Player {
        return Player {
            name,
            address
        }
    }

    /// Viewer function to read the private member of the Player struct.
    /// We could make the member public instead and access it directly if we wanted to simplify
    /// access patterns
    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn address(&self) -> &Addr {&self.address}
}

impl PartialEq for Player {
    fn eq(&self, other: &Self) -> bool {
        self.address == other.address
    }
}

pub fn config(storage: &mut dyn Storage) -> Singleton<State> {
    singleton(storage, CONFIG_KEY)
}

pub fn config_read(storage: &dyn Storage) -> ReadonlySingleton<State> {
    singleton_read(storage, CONFIG_KEY)
}

pub fn calculate_winner(p1: &RPS, p2: &RPS) -> GameResult {
    match (p1, p2) {
        (RPS::Rock, RPS::Rock) => {
            GameResult::Tie
        },
        (RPS::Rock, RPS::Paper) => {
            GameResult::Player2
        },
        (RPS::Rock, RPS::Scissors) => {
            GameResult::Player1
        },
        (RPS::Paper, RPS::Paper) => {
            GameResult::Tie
        },
        (RPS::Paper, RPS::Rock) => {
            GameResult::Player1
        },
        (RPS::Paper, RPS::Scissors) => {
            GameResult::Player2
        },
        (RPS::Scissors, RPS::Scissors) => {
            GameResult::Tie
        },
        (RPS::Scissors, RPS::Rock) => {
            GameResult::Player2
        },
        (RPS::Scissors, RPS::Paper) => {
            GameResult::Player1
        }
    }
}