use std::collections::HashMap;
use cosmwasm_std::{entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Coin, Uint128, StdError, BankMsg, Event};
use rand_core::RngCore;
use crate::contract::GameResult::Corner;

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
        CornerType::TopLeft => { GameResult::Corner {nums: (winner, winner - 1, winner - 3, winner - 1)}}
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

    if result == 0 {
        return winners;
    }

    winners.push(GameResult::Exact {num: result});

    if result % 2 == 0 {
        winners.push(GameResult::Even);
    } else {
        winners.push(GameResult::Odd);
    }

    if result >= 19 {
        winners.push(GameResult::Range19to36);
    } else {
        winners.push(GameResult::Range1to18);
    }

    match result % 3 {
        0 => {
            winners.push(GameResult::Range2to1Third);
            if result != 36 {
                winners.push(corner_result(result, CornerType::BottomLeft));
            }
            if result != 3 {
                winners.push(corner_result(result, CornerType::TopLeft));
            }

            winners.push(line_result(result, LineType::Left));

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

            winners.push(line_result(result, LineType::Right));
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

    match result {
        1 | 2 | 3 => {
            // line under
            winners.push(line_result(result, LineType::Under));
        },
        33 | 34 | 35 => {
            // both
            winners.push(line_result(result, LineType::Over));
        },
        _ => {
            // both
            winners.push(line_result(result, LineType::Over));
            winners.push(line_result(result, LineType::Under));
        }
    }

    match result {
        1 | 7 | 16 | 25 | 28 | 34 | 5 | 14 | 23 | 32 | 3 | 9 | 12 | 18| 21 | 27 | 30 | 36 => {
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
            let _ = item.checked_add(b.amount.amount);
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

    return true;
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

    for win_bet in winning_bets {
        let payout_amount = win_bet.amount.amount * Uint128::from(win_bet.result.payout());
        payouts.entry(win_bet.amount.denom).and_modify(|amount| *amount += payout_amount).or_insert(payout_amount);
    }

    let coins_to_send: Vec<Coin> = payouts.iter().map(|payout| Coin { denom: payout.0.to_string(), amount: payout.1.clone() }).collect();

    let resp = Response::new().add_event(Event::new("wasm-roulette_result").add_attribute_plaintext(
        "result", result.to_string()
    ));

    if coins_to_send.len() > 0 {
        deps.api.debug(&format!("payouts to send: {:?}", coins_to_send.clone()));

        let msg = BankMsg::Send { to_address: info.sender.to_string(), amount: coins_to_send };

        Ok(resp.add_message(msg))
    } else {
        Ok(resp)
    }
}


#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    Ok(Binary::default())
}

#[cfg(test)]
mod tests {
    use std::borrow::BorrowMut;
    use super::*;

    use cosmwasm_std::testing::{
        mock_dependencies, mock_env, mock_info, MockApi, MockQuerier, MockStorage,
    };
    use cosmwasm_std::{coins, OwnedDeps};

    /// Just set sender and funds for the message.
    /// This is intended for use in test code only.
    pub fn mock_info_random(sender: &str, funds: &[Coin]) -> MessageInfo {
        MessageInfo {
            sender: cosmwasm_std::Addr::unchecked(sender),
            funds: funds.to_vec(),
            random: Binary::from_base64("w6vk77ptGUl4u0cjkvFehZICh9UrYiT3HZJSfV5zY5k=").unwrap()
        }
    }

    fn instantiate_contract(deps: DepsMut) -> MessageInfo {
        let msg = InstantiateMsg {};
        let info = mock_info_random("creator", &coins(200, "token"));
        let _res = instantiate(deps, mock_env(), info.clone(), msg).unwrap();
        info
    }

    #[test]
    fn new_game() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        let info = instantiate_contract(deps.as_mut());

        let bet = Bet{ amount: Coin { denom: "token".to_string(), amount: Uint128::from(200 as u16) }, result: GameResult::Red };

        let execute_msg = ExecuteMsg::Bet {bets: vec![bet]};

        let res = execute(deps.as_mut(), mock_env(), info, execute_msg).unwrap();

        assert_eq!(res.messages.len(), 0)
    }

    #[test]
    fn new_game_winner() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        let info = instantiate_contract(deps.as_mut());

        let bet = Bet{ amount: Coin { denom: "token".to_string(), amount: Uint128::from(200 as u16) }, result: GameResult::Line {nums: (31, 28)} };

        let execute_msg = ExecuteMsg::Bet {bets: vec![bet]};

        let res = execute(deps.as_mut(), mock_env(), info, execute_msg).unwrap();

        assert_eq!(res.messages.len(), 1)
    }
}
