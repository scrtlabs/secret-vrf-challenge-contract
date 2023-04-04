
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
                num < &37u32
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

impl Into<String> for GameResult {
    fn into(self) -> String {
        match self {
            GameResult::Exact { num } => num.to_string(),
            GameResult::Red => "red".to_string(),
            GameResult::Black => "black".to_string(),
            GameResult::Range1to12 => "1-12".to_string(),
            GameResult::Range13to24 => "13-24".to_string(),
            GameResult::Range25to36 => "25-36".to_string(),
            GameResult::Odd => "odd".to_string(),
            GameResult::Even => "even".to_string(),
            GameResult::Range2to1First => "2to11st".to_string(),
            GameResult::Range2to1Second => "2to12nd".to_string(),
            GameResult::Range2to1Third => "2to13rd".to_string(),
            GameResult::Line { nums } => format!("double-{}-{}", nums.0, nums.1),
            GameResult::Corner { nums } => format!("quad-{},{},{},{}", nums.0, nums.1, nums.2, nums.3),
            GameResult::Range1to18 => "1-18".to_string(),
            GameResult::Range19to36 => "19-36".to_string(),
        }
    }
}

impl PartialEq for GameResult {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (GameResult::Line { nums: (other_a, other_b) }, GameResult::Line { nums: (this_a, this_b) }) => {
                (this_a == other_b || this_a == other_a) && (this_b == other_b || this_b == other_a)
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
        }

    }

    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
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
