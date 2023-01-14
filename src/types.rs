use std::any::Any;
use cosmwasm_std::Coin;
use serde::{Serialize, Deserialize};
use schemars::JsonSchema;

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Bet {
    pub amount: Coin,
    pub result: GameResult
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum GameResult {
    Exact {
        num: u32
    },
    Red,
    Black,
    Range1to12,
    Range13to24,
    Range25to36,
    Odd,
    Even,
    /// 1, 4, 7, 10...
    Range2to1First,
    /// 2, 5, 8, 11...
    Range2to1Second,
    /// 3, 6, 9, 12...
    Range2to1Third,
    Line {
        nums: (u32, u32)
    },
    Corner {
        nums: (u32, u32, u32, u32)
    },
    Range1to18,
    Range19to36,

}

impl GameResult {
    pub fn validate(&self) -> bool {
        match self {
            GameResult::Exact { num } => {
                return num < &37u32
            },
            GameResult::Line { nums: (n1, n2) } => {
                n1 != n2
            },
            GameResult::Corner { nums: (n1, n2, n3, n4) } => {
                // too lazy to actually validate the values
                n1 != n2 && n1 != n3 && n1 != n4 && n2 != n3 && n2 != n4 && n3 != n4
            },
            _ => true,
        }
    }
}

impl PartialEq for GameResult {
    fn eq(&self, other: &Self) -> bool {
        return match (self, other) {
            (GameResult::Line { nums: (other_a, other_b) }, GameResult::Line { nums: (this_a, this_b) }) => {
                if (this_a == other_b || this_a == other_a) && (this_b == other_b || this_b == other_a) {
                    true
                } else {
                    false
                }
            }
            (GameResult::Corner { nums: (other_a, other_b, other_c, other_d) },
                GameResult::Corner { nums: (this_a, this_b, this_c, this_d) }
            ) => {
                let temp_v = vec![other_a, other_b, other_c, other_d];

                for num in [this_a, this_b, this_c, this_d] {
                    if !temp_v.contains(&num) {
                        return false;
                    }
                }

                true
            }
            (GameResult::Black, GameResult::Black) => true,
            (GameResult::Red, GameResult::Red) => true,
            (GameResult::Range1to18, GameResult::Range1to18) => true,
            (GameResult::Range19to36, GameResult::Range19to36) => true,
            (GameResult::Range2to1Third, GameResult::Range2to1Third) => true,
            (GameResult::Range2to1Second, GameResult::Range2to1Second) => true,
            (GameResult::Range2to1First, GameResult::Range2to1First) => true,
            (GameResult::Odd, GameResult::Odd) => true,
            (GameResult::Even, GameResult::Even) => true,
            (GameResult::Range1to12, GameResult::Range1to12) => true,
            (GameResult::Range13to24, GameResult::Range13to24) => true,
            (GameResult::Range25to36, GameResult::Range25to36) => true,
            (GameResult::Exact { num: n1 }, GameResult::Exact { num: n2 }) => {
                n1 == n2
            },
            _ => false,
        };

    }

    fn ne(&self, other: &Self) -> bool {
        todo!()
    }
}

impl GameResult {
    pub fn payout(&self) -> u8 {
        match self {
            GameResult::Exact { .. } => 36,
            GameResult::Red => 2,
            GameResult::Black => 2,
            GameResult::Range1to12 => 3,
            GameResult::Range13to24 => 3,
            GameResult::Range25to36 => 3,
            GameResult::Odd => 2,
            GameResult::Even => 2,
            GameResult::Range2to1First => 3,
            GameResult::Range2to1Second => 3,
            GameResult::Range2to1Third => 3,
            GameResult::Line { .. } => 18,
            GameResult::Corner { .. } => 9,
            GameResult::Range1to18 => 2,
            GameResult::Range19to36 => 2,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, JsonSchema)]
pub enum CornerType {
    BottomLeft = 0,
    BottomRight,
    TopLeft,
    TopRight
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, JsonSchema)]
pub enum LineType {
    Over = 0,
    Under,
    Left,
    Right
}
