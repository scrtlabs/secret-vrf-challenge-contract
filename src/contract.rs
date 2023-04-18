use std::collections::HashMap;
use cosmwasm_std::{entry_point, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Coin, Uint128, StdError, BankMsg, Event};
use cosmwasm_std::CosmosMsg::LastMsgMark;
use rand_core::RngCore;


use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::rng::Prng;
use crate::types::{Bet, CornerType, GameResult, LineType};

#[entry_point]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> StdResult<Response> {

    // save init params to state

    Ok(Response::default())
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, StdError> {

    match msg {
        ExecuteMsg::Bet { bets } =>
            handle_game_result(deps, env, info, bets)
    }
}


fn corner_result(winner: u32, corner: CornerType) -> GameResult {
    match corner {
        CornerType::BottomLeft => { GameResult::Corner {nums: (winner, winner + 3, winner - 1, winner + 2)}}
        CornerType::BottomRight => { GameResult::Corner {nums: (winner, winner + 3, winner + 1, winner + 4)}}
        CornerType::TopLeft => { GameResult::Corner {nums: (winner, winner - 1, winner - 3, winner - 4)}}
        CornerType::TopRight => { GameResult::Corner {nums: (winner, winner + 1, winner - 3, winner - 2)}}
    }
}

fn line_result(winner: u32, line: LineType) -> GameResult {
    match line {
        LineType::Over => { GameResult::Line { nums: (winner, winner - 3) }}
        LineType::Under => { GameResult::Line { nums: (winner, winner + 3) }}
        LineType::Left => { GameResult::Line { nums: (winner, winner + 1) }}
        LineType::Right => { GameResult::Line { nums: (winner, winner - 1) }}
    }
}

fn return_winning_numbers(result: u32) -> Vec<GameResult> {

    if result > 36 {
        panic!("Can never happen")
    }

    let mut winners: Vec<GameResult> = vec![];



    // exact number
    winners.push(GameResult::Exact {num: result});

    if result == 0 {
        return winners;
    }

    // odd or even
    if result % 2 == 0 {
        winners.push(GameResult::Even);
    } else {
        winners.push(GameResult::Odd);
    }

    // upper or lower
    if result >= 19 {
        winners.push(GameResult::Range19to36);
    } else {
        winners.push(GameResult::Range1to18);
    }

    // corners and lines
    match result % 3 {
        0 => {
            winners.push(GameResult::Range2to1Third);
            if result != 36 {
                winners.push(corner_result(result, CornerType::BottomLeft));
            }
            if result != 3 {
                winners.push(corner_result(result, CornerType::TopLeft));
            }

            winners.push(line_result(result, LineType::Right));
        }
        1 => {
            winners.push(GameResult::Range2to1First);
            if result != 34 {
                // bottom right
                winners.push(corner_result(result, CornerType::BottomRight));
            }
            if result != 1 {
                // top right
                winners.push(corner_result(result, CornerType::TopRight));
            }
            winners.push(line_result(result, LineType::Left));
        }
        2 => {
            winners.push(GameResult::Range2to1Second);

            if result != 2 {
                winners.push(corner_result(result, CornerType::TopRight));
                winners.push(corner_result(result, CornerType::TopLeft));
            }

            if result != 35 {
                winners.push(corner_result(result, CornerType::BottomLeft));
                winners.push(corner_result(result, CornerType::BottomRight));
            }

            winners.push(line_result(result, LineType::Right));
            winners.push(line_result(result, LineType::Left));

        }

        _ => {panic!("Not possible")}
    }

    // under/over lines
    match result {
        1 | 2 | 3 => {
            // line under
            winners.push(line_result(result, LineType::Under));
        },
        34 | 35 | 36 => {
            // both
            winners.push(line_result(result, LineType::Over));
        },
        _ => {
            // both
            winners.push(line_result(result, LineType::Over));
            winners.push(line_result(result, LineType::Under));
        }
    }

    // 1-12/13-24/25-36
    match result {
        1..=12 => winners.push(GameResult::Range1to12),
        13..=24 => winners.push(GameResult::Range13to24),
        25..=36 => winners.push(GameResult::Range25to36),
        _ => unreachable!(),
    };

    // red or black - american table
    match result {
        1 | 3 | 5 | 7 | 9 | 12 | 14 | 16 | 18 | 19 | 21 | 23 | 25 | 27 | 30 | 32 | 34 | 36 => {
            winners.push(GameResult::Red)
        },
        _ => {
            winners.push(GameResult::Black)
        }
    }

    winners
}

fn calculate_sum_coins_of_bets(bets: &Vec<Bet>) -> HashMap<String, Uint128> {
    let mut coins: std::collections::HashMap<String, Uint128> = std::collections::HashMap::default();
    for b in bets {
        let this_item = coins.get_mut(&b.amount.denom);

        if this_item.is_none() {
            coins.insert(b.amount.denom.clone(), b.amount.amount);
        } else {
            let item = this_item.unwrap();
            if let Ok(result) = item.checked_add(b.amount.amount) {
                *item = result;
            } else {
                panic!("Overflow when adding coins");
            }
        }
    }

    coins
}

fn check_coins_match_input(coins: HashMap<String, Uint128>, sent_funds: Vec<Coin>) -> bool {
    for funds in sent_funds {
        if coins.get(&funds.denom).unwrap_or(&Uint128::zero()) != &funds.amount {
            return false;
        }
    }

    true
}

fn handle_game_result(deps: DepsMut, env: Env, info: MessageInfo, bets: Vec<Bet>) -> Result<Response, StdError> {

    deps.api.debug(&format!("Bets are in: {:?}", bets));

    let sums = calculate_sum_coins_of_bets(&bets);

    if !check_coins_match_input(sums, info.funds) {
        return Err(StdError::generic_err("Input funds don't match sum of bets"));
    }

    for b in &bets {
        if !b.result.validate() {
            deps.api.debug(&format!("Invalid bet dawg: {:?}", b.result));
            return Err(StdError::generic_err("Error, invalid bet"));
        }
    }

    if env.block.random.is_none() {
        return Err(StdError::generic_err("Error, random not available"));
    }
    let r: Binary = env.block.random.unwrap();

    let mut prng = Prng::new(r.as_slice());

    // this is probably fine since the modulo bias is super small
    let result = prng.next_u32() % 37;

    deps.api.debug(&format!("Roll result: {:?}", result));

    let winners = return_winning_numbers(result);

    deps.api.debug(&format!("Winning bets are: {:?}", winners));

    let mut winning_bets = vec![];
    for bet in bets {
        if winners.contains(&bet.result) {
            winning_bets.push(bet)
        }
    }

    let mut payouts: HashMap<String, Uint128> = std::collections::HashMap::new();

    deps.api.debug(&format!("payouts for bets are: {:?}", payouts));


    let mut winning_bets_evt = Event::new("winners");

    for win_bet in winning_bets {
        let payout_amount = win_bet.amount.amount * Uint128::from(win_bet.result.payout());

        winning_bets_evt = winning_bets_evt.add_attribute_plaintext(win_bet.result.clone(), payout_amount);

       // winning_bets_attrs.push(Attribute::new(win_bet.result.clone(), payout_amount));

        payouts.entry(win_bet.amount.denom).and_modify(|amount| *amount += payout_amount).or_insert(payout_amount);

    }

    let coins_to_send: Vec<Coin> = payouts.iter().map(|payout| Coin { denom: payout.0.to_string(), amount: *payout.1 }).collect();

    let resp = Response::new().add_event(Event::new("wasm-roulette_result").add_attribute_plaintext(
        "result", result.to_string()
    ));

    if !coins_to_send.is_empty() {
        deps.api.debug(&format!("payouts to send: {:?}", coins_to_send));

        let msg = BankMsg::Send { to_address: info.sender.to_string(), amount: coins_to_send };

        Ok(resp
            .add_event(winning_bets_evt)
            .add_message(msg)
            .add_message(
                LastMsgMark(cosmwasm_std::Empty {})
            )

        )
    } else {
        Ok(resp)
    }
}


#[entry_point]
pub fn query(_deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
    Ok(Binary::default())
}

#[cfg(test)]
mod tests {
    
    use super::*;

    use cosmwasm_std::testing::{
        mock_dependencies, mock_env, mock_info,
    };
    use cosmwasm_std::{coins};
    use std::collections::HashMap;
    use crate::contract::return_winning_numbers;
    /// Just set sender and funds for the message.
    /// This is intended for use in test code only.

    fn instantiate_contract(deps: DepsMut) -> MessageInfo {
        let msg = InstantiateMsg {};
        let info = mock_info("creator", &coins(200, "token"));
        let _res = instantiate(deps, mock_env(), info.clone(), msg).unwrap();
        info
    }

    #[test]
    fn new_game_loser() {
        let mut deps = mock_dependencies();
        let _env = mock_env();

        let info = instantiate_contract(deps.as_mut());

        let bet = Bet{ amount: Coin { denom: "token".to_string(), amount: Uint128::from(200_u16) }, result: GameResult::Black };

        let execute_msg = ExecuteMsg::Bet {bets: vec![bet]};

        let res = execute(deps.as_mut(), mock_env(), info, execute_msg).unwrap();

        assert_eq!(res.messages.len(), 0)
    }

    #[test]
    fn new_game_winner() {
        let mut deps = mock_dependencies();
        let _env = mock_env();

        let info = instantiate_contract(deps.as_mut());

        let bet = Bet{ amount: Coin { denom: "token".to_string(), amount: Uint128::from(200_u16) }, result: GameResult::Line {nums: (1, 2)} };

        let execute_msg = ExecuteMsg::Bet {bets: vec![bet]};

        let res = execute(deps.as_mut(), mock_env(), info, execute_msg).unwrap();

        assert_eq!(res.messages.len(), 1)
    }


    #[test]
    fn test_return_winning_numbers() {
        let mut test_cases: HashMap<u32, Vec<GameResult>> = HashMap::new();

        test_cases.insert(0, vec![
            GameResult::Exact { num: 0 },
        ]);

        for i in 1..=36 {
            let red_numbers = [
                1, 3, 5, 7, 9, 12, 14, 16, 18, 19, 21, 23, 25, 27, 30, 32, 34, 36,
            ];
            let is_red = red_numbers.contains(&i);
            let is_even = i % 2 == 0;
            let range = match i {
                1..=12 => GameResult::Range1to12,
                13..=24 => GameResult::Range13to24,
                25..=36 => GameResult::Range25to36,
                _ => unreachable!(),
            };

            let range2to1 = match i % 3 {
                1 => GameResult::Range2to1First,
                2 => GameResult::Range2to1Second,
                0 => GameResult::Range2to1Third,
                _ => unreachable!(),
            };

            let lines: Vec<GameResult> = {
                let mut v = Vec::new();

                if i < 36 && i % 3 != 0 {
                    v.push(GameResult::Line { nums: (i, i + 1) });
                }
                if i > 3 {
                    v.push(GameResult::Line { nums: (i - 3, i) });
                }
                if i < 34 {
                    v.push(GameResult::Line { nums: (i, i + 3) });
                }
                if i > 1 && i % 3 != 1 {
                    v.push(GameResult::Line { nums: (i - 1, i) });
                }

                v
            };

            let corners: Vec<GameResult> = {
                let mut v = Vec::new();

                if i % 3 != 0 && i <= 33 {
                    v.push(GameResult::Corner {
                        nums: (i, i + 1, i + 3, i + 4),
                    });
                }
                if i % 3 != 1 && i <= 33 {
                    v.push(GameResult::Corner {
                        nums: (i, i - 1, i + 3, i + 2),
                    });
                }
                if i % 3 != 0 && i >= 4 {
                    v.push(GameResult::Corner {
                        nums: (i, i + 1, i - 3, i - 2),
                    });
                }
                if i % 3 != 1 && i >= 5 {
                    v.push(GameResult::Corner {
                        nums: (i, i - 1, i - 3, i - 4),
                    });
                }

                v
            };

            let mut outcomes = vec![
                GameResult::Exact { num: i },
                range,
                if is_red { GameResult::Red } else { GameResult::Black },
                if is_even { GameResult::Even } else { GameResult::Odd },
                range2to1,
            ];

            outcomes.extend(lines);
            outcomes.extend(corners);

            if i <= 18 {
                outcomes.push(GameResult::Range1to18);
            } else {
                outcomes.push(GameResult::Range19to36);
            }

            test_cases.insert(i, outcomes);
        }

        // Fill the test_cases HashMap with all possible roulette results
        // and their expected GameResult outcomes.
        // Note: This is just an example, you should add all the possible test cases here
        // test_cases.insert(1, vec![
        //     GameResult::Exact { num: 1 },
        //     GameResult::Red,
        //     GameResult::Range1to12,
        //     GameResult::Odd,
        //     GameResult::Range2to1First,
        //     GameResult::Line { nums: (1, 2) },
        //     GameResult::Corner { nums: (1, 2, 4, 5) },
        //     GameResult::Range1to18,
        // ]);

        // the test code was written by ChatGPT - instead of doing all the manual labor of going through
        // everything manually, letting the AI write the test code, then making sure both codes return
        // the same results should cover all the outliers

        // Iterate through test_cases and check if return_winning_numbers
        // provides the correct results
        for (roll_result, expected_outcomes) in test_cases.iter() {
            let winning_numbers = return_winning_numbers(*roll_result);
            if winning_numbers != *expected_outcomes {

                let differences = winning_numbers.iter().filter(
                    |&g| !expected_outcomes.contains(g)
                ).collect::<Vec<&GameResult>>();
                if !differences.is_empty() {
                    println!("Test case failed:");
                    println!("\tExpected from test: {:?}", expected_outcomes);
                    println!("\tReturn from contract: {:?}", winning_numbers);
                    println!("\tDifferences: \n\t\t{:?}", differences);
                    panic!("There are differences between expected and returned results");
                }

                let differences2 = expected_outcomes.iter().filter(
                    |&g| !winning_numbers.contains(g)
                ).collect::<Vec<&GameResult>>();
                if !differences2.is_empty() {
                    println!("Test case failed:");
                    println!("\tExpected from test: {:?}", expected_outcomes);
                    println!("\tReturn from contract: {:?}", winning_numbers);
                    println!("\tDifferences: \n\t\t{:?}", differences2);
                    panic!("There are differences between expected and returned results");
                }
            }
        }
    }

    use cosmwasm_std::{StdError, Uint128};

    #[test]
    fn test_calculate_sum_coins_of_bets_and_check_coins_match_input() {
        let bets = vec![
            Bet {
                amount: Coin::new(10, "abc"),
                result: GameResult::Exact { num: 13 },
            },
            Bet {
                amount: Coin::new( 5, "def"),
                result: GameResult::Red,
            },
            Bet {
                amount: Coin::new( 5, "abc"),
                result: GameResult::Range1to12,
            },
        ];

        let funds = vec![
            Coin::new( 20, "abc"),
            Coin::new( 5, "def"),
        ];

        let sum_of_bets = calculate_sum_coins_of_bets(&bets);

        let result = check_coins_match_input(calculate_sum_coins_of_bets(&bets), funds.clone());
        assert_eq!(result, false);

        let bets = vec![
            Bet {
                amount: Coin::new( 10, "abc"),
                result: GameResult::Exact { num: 13 },
            },
            Bet {
                amount: Coin::new( 5, "def"),
                result: GameResult::Red,
            },
            Bet {
                amount: Coin::new(5, "abc"),
                result: GameResult::Range1to12,
            },
        ];

        let funds = vec![
            Coin::new(15, "abc"),
            Coin::new( 5, "def"),
        ];

        let sum_of_bets = calculate_sum_coins_of_bets(&bets);

        let result = check_coins_match_input(calculate_sum_coins_of_bets(&bets), funds.clone());
        assert_eq!(result, true);
        //assert_eq!(result.unwrap_err(), StdError::generic_err("Input funds don't match sum of bets"));
    }
}
