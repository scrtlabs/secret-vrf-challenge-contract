use std::cmp::max;
use cosmwasm_std::{entry_point, to_binary, Deps, DepsMut, Env, MessageInfo, QueryResponse, Response, StdError, StdResult, Uint128, Coin, CosmosMsg, BankMsg, attr, SubMsg};

use crate::errors::{CustomContractError};
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, CheckWinner};
use crate::state::{config, config_read, CurrentStatus, GameResult, GameState, Player, RPS, State};

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> StdResult<Response> {

    let state = State::default();
    config(deps.storage).save(&state)?;

    Ok(Response::default())
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, CustomContractError> {
    match msg {
        ExecuteMsg::NewGame { bet, name } => try_new_game(deps, info, bet, name),
        ExecuteMsg::SubmitChoice {game, choice} => try_submit_choice(deps, info, env, game, choice),
        ExecuteMsg::Finalize { game } => try_finalize(deps, env, game),
        ExecuteMsg::JoinGame { game } => try_join(deps, info, game),
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
    game: String
) -> Result<Response, CustomContractError> {

    let mut state = config(deps.storage).load()?;

    if let Some(some_bet) = state.bet {
        if !info.funds.contains(&some_bet) {
            return Err(CustomContractError::Std(StdError::generic_err("Sent funds do not match the proposed bet")));
        }
    }


    Ok(Response::new())
}

pub fn try_finalize(
    deps: DepsMut,
    env: Env,
    game: String
) -> Result<Response, CustomContractError> {

    let mut state = config(deps.storage).load()?;

    match state.game_state.status {
        CurrentStatus::DoneGettingChoices => {},
        _ => {return Err(CustomContractError::Std((StdError::generic_err("Cannot finalize right now"))))}
    }

    if env.block.height > state.game_state.end_game_block.unwrap() {
        return Err(CustomContractError::Std((StdError::generic_err("Cannot finalize right now"))))
    }

    // todo: check if winner exists
    let winner = state.game_state.winner.unwrap();
    
    let mut messages = vec![];

    let winnings = Coin { denom: state.bet.denom, amount: Uint128(state.bet.amount.u128() * 2) };

    match winner {
        GameResult::Player1 => {
            messages.push(SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
                to_address: state.players[0].address().to_string(),
                amount: vec![winnings]
            })));
        }
        GameResult::Player2 => {
            messages.push(SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
                to_address: state.players[1].address().to_string(),
                amount: vec![winnings]
            })));
        }
        GameResult::Tie => {}
    }

    state.next();

    config(deps.storage).save(&state)?;

    Ok(Response {
        messages,
        attributes: vec![],
        events: vec![],
        data: None
    })
}

pub fn try_submit_choice(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    game: String,
    choice: RPS,
) -> Result<Response, CustomContractError> {
    let mut state = config(deps.storage).load()?;

    match state.game_state.status {
        CurrentStatus::Started | CurrentStatus::Got1stChoice => {},
        _ => {return Err(CustomContractError::Std((StdError::generic_err("Cannot submit choice right now"))))}
    }

    _set_choice_for_player(info, choice, &mut state)?;
    state.game_state.status.next();
    
    if state.game_state.status == CurrentStatus::DoneGettingChoices {
        state.game_state.end_game_block = Some(env.block.height);
    }

    config(deps.storage).save(&state)?;

    Ok(Response::new())
}

fn _set_choice_for_player(info: MessageInfo, choice: RPS, state: &mut State) -> Result<(), CustomContractError> {
    if info.sender == state.players[0].address() {
        state.choices[0] = Some(choice);
    } else if info.sender == state.players[1].address() {
        state.choices[1] = Some(choice);
    } else {
        return Err(CustomContractError::Std((StdError::generic_err("Sender is not in this game"))));
    }

    Ok(())
}

pub fn try_new_game(
    deps: DepsMut,
    info: MessageInfo,
    bet: Option<Coin>,
    name: String,
) -> Result<Response, CustomContractError> {

    let mut state = State::default();

    if let Some(some_bet) = bet {
        if !info.funds.contains(&some_bet) {
            return Err(CustomContractError::Std(StdError::generic_err("Sent funds do not match the proposed bet")));
        }
        state.bet = Some(some_bet);
    }

    state.next();
    state.players[0] = Player::new(name, info.sender);

    config(deps.storage).save(&state)?;

    Ok(Response {
        messages: vec![],
        attributes: vec![],
        events: vec![cosmwasm_std::Event { ty: "new_rps_game".to_string(), attributes: vec![attr("game_code", "AAAA")] }],
        data: None
    })
}

pub fn try_reset(
    deps: DepsMut,
) -> Result<Response, CustomContractError> {
    let mut state = config(deps.storage).load()?;

    state.game_state = GameState::Init;
    config(deps.storage).save(&state)?;

    Ok(Response::new()
        .add_attribute("action", "reset state"))
}

fn query_who_won(deps: Deps, env: Env, game: String) -> StdResult<CheckWinner> {
    let state = config_read(deps.storage).load()?;

    if state.game_state.status != GameState::GotFromPlayer2 {
        return Err(StdError::generic_err("Players didn't finish submitting their choice!"));
    }

    if state.game_state.end_game_block.unwrap_or(0) <= env.block.height {
        return Err(StdError::generic_err("Still processing results!"));
    }

    return Ok(match state.game_state.winner.unwrap_or_default() {
        GameResult::Player1 => { CheckWinner { winner: GameResult::Player1, address: Some(state.player1.address().clone()) } },
        GameResult::Player2 => { CheckWinner { winner: GameResult::Player2, address: Some(state.player2.address().clone()) } },
        GameResult::Tie => { CheckWinner { winner: GameResult::Tie, address: None } }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    use cosmwasm_std::testing::{mock_env, mock_info, mock_dependencies};
    use cosmwasm_std::coins;

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

        let msg_player1 = ExecuteMsg::SubmitNetWorth {worth: 1, name: "alice".to_string()};
        let msg_player2 = ExecuteMsg::SubmitNetWorth {worth: 2, name: "bob".to_string()};

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

        let msg_player1 = ExecuteMsg::SubmitNetWorth {worth: 1, name: "alice".to_string()};

        let info = mock_info("creator", &[]);
        let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg_player1).unwrap();

        let reset_msg = ExecuteMsg::Reset {};
        let _res = execute(deps.as_mut(), mock_env(), info.clone(), reset_msg).unwrap();

        let msg_player2 = ExecuteMsg::SubmitNetWorth {worth: 2, name: "bob".to_string()};
        let msg_player3 = ExecuteMsg::SubmitNetWorth {worth: 3, name: "carol".to_string()};

        let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg_player2).unwrap();
        let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg_player3).unwrap();

        // it worked, let's query the state
        let value = query_who_won(deps.as_ref()).unwrap();

        assert_eq!(&value.richer, "carol")    }
}
