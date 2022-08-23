use cosmwasm_std::{Addr, Coin, StdResult, Storage};
use cw_storage_plus::Map;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum RPS {
    Rock,
    Paper,
    Scissors,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct State {
    pub game_state: GameState,
    pub players: [Option<Player>; 2],
    pub choices: [Option<RPS>; 2],
    pub bet: Option<Coin>,
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug, Default)]
pub struct GameState {
    pub status: CurrentStatus,
    pub end_game_block: Option<u64>,
    pub winner: Option<GameResult>,
}

impl State {
    pub fn next(&mut self) {
        // make sure to check the status before advancing it
        if &self.game_state.status == &CurrentStatus::Got1stChoice {
            if let (Some(choice1), Some(choice2)) = (&self.choices[0], &self.choices[1]) {
                self.game_state.winner = Some(calculate_winner(choice1, choice2));
            }
        }
        self.game_state.status.next();
    }
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub enum GameResult {
    Player1,
    Player2,
    Tie,
}

#[derive(Serialize, Deserialize, Copy, PartialEq, Clone, Debug)]
pub enum CurrentStatus {
    Initialized = 0,
    WaitingForPlayerToJoin,
    Started,
    Got1stChoice,
    DoneGettingChoices,
    Finalized,
}

impl CurrentStatus {
    pub(crate) fn next(&mut self) {
        match self {
            Self::Initialized => *self = Self::WaitingForPlayerToJoin,
            Self::WaitingForPlayerToJoin => *self = Self::Started,
            Self::Started => *self = Self::Got1stChoice,
            Self::Got1stChoice => *self = Self::DoneGettingChoices,
            Self::DoneGettingChoices => *self = Self::Finalized,
            Self::Finalized => *self = Self::Started,
        }
    }

    // fn reset(mut self) {
    //     self = Self::Initialized;
    // }
}

impl Default for CurrentStatus {
    fn default() -> Self {
        Self::Initialized
    }
}

// impl From<u8> for CurrentStatus {
//     fn from(num: u8) -> Self {
//         match num {
//             0 => CurrentStatus::Initialized,
//             1 => CurrentStatus::Started,
//             2 => CurrentStatus::GotFromPlayer2,
//             3 => CurrentStatus::Finalized,
//             _ => CurrentStatus::Init
//         }
//     }
// }
//
// impl From<CurrentStatus> for u8 {
//     fn from(state: CurrentStatus) -> Self {
//         match state {
//             CurrentStatus:: => 0,
//             CurrentStatus::GotFromPlayer1 => 1,
//             CurrentStatus::GotFromPlayer2 => 2,
//             CurrentStatus::Finalized => 3
//         }
//     }
// }

#[derive(Serialize, Deserialize, Clone, Debug, Eq)]
pub struct Player {
    name: String,
    address: Addr,
}

impl Player {
    /// Constructor function. Takes input parameters and initializes a struct containing both
    /// those items
    pub fn new(name: String, address: Addr) -> Player {
        return Player { name, address };
    }

    /// Viewer function to read the private member of the Player struct.
    /// We could make the member public instead and access it directly if we wanted to simplify
    /// access patterns
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

pub fn save_game_state(storage: &mut dyn Storage, game: &str, state: State) -> StdResult<()> {
    const GAME_STATE: Map<&[u8], State> = Map::new("game_state");
    GAME_STATE.save(storage, game.as_bytes(), &state)
}

pub fn load_game_state(storage: &dyn Storage, game: &str) -> StdResult<State> {
    const GAME_STATE: Map<&[u8], State> = Map::new("game_state");
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
