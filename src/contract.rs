use cosmwasm_std::{
    entry_point, to_binary, BankMsg, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo,
    QueryResponse, Response, StdError, StdResult, Uint128,
};

use crate::errors::CustomContractError;
use crate::errors::CustomContractError::Std;
use crate::msg::{CheckWinner, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{
    load_game_state, save_game_state, CurrentStatus, GameResult, Player, State, RPS,
};

#[entry_point]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> StdResult<Response> {
    Ok(Response::default())
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, CustomContractError> {
    match msg {
        ExecuteMsg::NewGame { bet, name } => try_new_game(deps, info, bet, name),
        ExecuteMsg::SubmitChoice { game, choice } => {
            try_submit_choice(deps, info, env, game, choice)
        }
        ExecuteMsg::Finalize { game } => try_finalize(deps, env, game),
        ExecuteMsg::JoinGame { name, game } => try_join(deps, info, name, game),
    }
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<QueryResponse> {
    match msg {
        QueryMsg::WhoWon { game } => to_binary(&query_who_won(deps, env, game)?),
    }
}

pub fn try_join(
    deps: DepsMut,
    info: MessageInfo,
    name: String,
    game: String,
) -> Result<Response, CustomContractError> {
    let mut state = load_game_state(deps.storage, &game)?;

    if let Some(some_bet) = &state.bet {
        if !info.funds.contains(&some_bet) {
            return Err(Std(StdError::generic_err(
                "Sent funds do not match the proposed bet",
            )));
        }
    }

    state.players[1] = Some(Player::new(name, info.sender));
    state.next();

    save_game_state(deps.storage, &game, state)?;

    Ok(Response::new())
}

pub fn try_finalize(
    deps: DepsMut,
    env: Env,
    game: String,
) -> Result<Response, CustomContractError> {
    let mut state = load_game_state(deps.storage, &game)?;

    match state.game_state.status {
        CurrentStatus::DoneGettingChoices => {}
        _ => return Err(Std(StdError::generic_err("Cannot finalize right now"))),
    }

    if env.block.height > state.game_state.end_game_block.unwrap() {
        return Err(Std(StdError::generic_err("Cannot finalize right now")));
    }

    let mut messages = vec![];

    if let (Some(bet), Some(winner)) = (&state.bet, &state.game_state.winner) {
        let winnings = Coin {
            denom: bet.denom.clone(),
            amount: Uint128::new(bet.amount.u128() * 2),
        };

        match winner {
            GameResult::Player1 => {
                if let Some(winning_player) = &state.players[0] {
                    messages.push(CosmosMsg::Bank(BankMsg::Send {
                        to_address: winning_player.address().to_string(),
                        amount: vec![winnings],
                    }));
                }
            }
            GameResult::Player2 => {
                if let Some(winning_player) = &state.players[1] {
                    messages.push(CosmosMsg::Bank(BankMsg::Send {
                        to_address: winning_player.address().to_string(),
                        amount: vec![winnings],
                    }));
                }
            }
            GameResult::Tie => {}
        }
    }

    state.next();

    save_game_state(deps.storage, &game, state)?;

    // fuck this syntax
    Ok(Response::default().add_messages(messages))
}

pub fn try_submit_choice(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    game: String,
    choice: RPS,
) -> Result<Response, CustomContractError> {
    let mut state = load_game_state(deps.storage, &game)?;

    match state.game_state.status {
        CurrentStatus::Started | CurrentStatus::Got1stChoice => {}
        _ => return Err(Std(StdError::generic_err("Cannot submit choice right now"))),
    }

    _set_choice_for_player(info, choice, &mut state)?;
    state.game_state.status.next();

    if state.game_state.status == CurrentStatus::DoneGettingChoices {
        state.game_state.end_game_block = Some(env.block.height);
    }

    save_game_state(deps.storage, &game, state)?;

    Ok(Response::new())
}

fn _set_choice_for_player(
    info: MessageInfo,
    choice: RPS,
    state: &mut State,
) -> Result<(), CustomContractError> {
    if let Some(player) = &state.players[0] {
        if &info.sender == player.address() {
            state.choices[0] = Some(choice);
        }
    } else if let Some(player) = &state.players[1] {
        if &info.sender == player.address() {
            state.choices[1] = Some(choice);
        }
    } else {
        return Err(Std(StdError::generic_err("Sender is not in this game")));
    }

    Ok(())
}

fn get_random_game_id() -> String {
    return "aaaa".to_string();
}

pub fn try_new_game(
    deps: DepsMut,
    info: MessageInfo,
    bet: Option<Coin>,
    name: String,
) -> Result<Response, CustomContractError> {
    let mut state = State::default();

    let game = get_random_game_id();

    if let Some(some_bet) = bet {
        if !info.funds.contains(&some_bet) {
            return Err(Std(StdError::generic_err(
                "Sent funds do not match the proposed bet",
            )));
        }
        state.bet = Some(some_bet);
    }

    state.next();
    state.players[0] = Option::from(Player::new(name, info.sender));

    save_game_state(deps.storage, &game, state)?;

    let resp = Response::new();

    let new_evt =
        cosmwasm_std::Event::new("new_rps_game".to_string()).add_attribute("game_code", "AAAA");

    Ok(resp.add_events(vec![new_evt]))
}

// pub fn try_reset(
//     deps: DepsMut,
// ) -> Result<Response, CustomContractError> {
//     let mut state = config(deps.storage).load()?;
//
//     state.game_state = GameState::default();
//     config(deps.storage).save(&state)?;
//
//     Ok(Response::new()
//         .add_attribute("action", "reset state"))
// }

fn query_who_won(deps: Deps, env: Env, game: String) -> StdResult<CheckWinner> {
    let state = load_game_state(deps.storage, &game)?;

    if state.game_state.status != CurrentStatus::Finalized {
        return Err(StdError::generic_err(
            "Players didn't finish submitting their choice!",
        ));
    }

    if state.game_state.end_game_block.unwrap_or(0) <= env.block.height {
        return Err(StdError::generic_err("Still processing results!"));
    }

    return Ok(match state.game_state.winner.unwrap() {
        GameResult::Player1 => {
            if let Some(winner) = &state.players[0] {
                CheckWinner {
                    winner: GameResult::Player1,
                    address: Some(winner.address().clone()),
                }
            } else {
                return Err(StdError::generic_err(
                    "Player 1 is the winner but undefined??",
                ));
            }
        }
        GameResult::Player2 => {
            if let Some(winner) = &state.players[1] {
                CheckWinner {
                    winner: GameResult::Player2,
                    address: Some(winner.address().clone()),
                }
            } else {
                return Err(StdError::generic_err(
                    "Player 2 is the winner but undefined??",
                ));
            }
        }
        GameResult::Tie => CheckWinner {
            winner: GameResult::Tie,
            address: None,
        },
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    use cosmwasm_std::coins;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};

    #[test]
    fn proper_instantialization() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {};
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let _ = query_who_won(deps.as_ref()).unwrap_err();
    }

    #[test]
    fn solve_millionaire() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {};
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let msg_player1 = ExecuteMsg::SubmitNetWorth {
            worth: 1,
            name: "alice".to_string(),
        };
        let msg_player2 = ExecuteMsg::SubmitNetWorth {
            worth: 2,
            name: "bob".to_string(),
        };

        let info = mock_info("creator", &[]);

        let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg_player1).unwrap();
        let _res = execute(deps.as_mut(), mock_env(), info, msg_player2).unwrap();

        // it worked, let's query the state
        let value = query_who_won(deps.as_ref()).unwrap();

        assert_eq!(&value.richer, "bob")
    }

    #[test]
    fn test_reset_state() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {};
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let msg_player1 = ExecuteMsg::SubmitNetWorth {
            worth: 1,
            name: "alice".to_string(),
        };

        let info = mock_info("creator", &[]);
        let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg_player1).unwrap();

        let reset_msg = ExecuteMsg::Reset {};
        let _res = execute(deps.as_mut(), mock_env(), info.clone(), reset_msg).unwrap();

        let msg_player2 = ExecuteMsg::SubmitNetWorth {
            worth: 2,
            name: "bob".to_string(),
        };
        let msg_player3 = ExecuteMsg::SubmitNetWorth {
            worth: 3,
            name: "carol".to_string(),
        };

        let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg_player2).unwrap();
        let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg_player3).unwrap();

        // it worked, let's query the state
        let value = query_who_won(deps.as_ref()).unwrap();

        assert_eq!(&value.richer, "carol")
    }
}
