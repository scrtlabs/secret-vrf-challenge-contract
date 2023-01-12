use crate::errors::CustomContractError;
use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};
use rand_core::RngCore;

use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, ReadShareResponse};
use crate::rng::Prng;

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
) -> Result<Response, CustomContractError> {
    match msg {
        ExecuteMsg::Bet { guess: u32 } =>
            bet(deps, env, info, guess: u32)
    }
}



const Table = vec![

];

enum GameResult {
    Color,
    OddOrEven,
    Line,
    Corner,
    Dozens,
    Thirds,
    HighOrLow,
    Exact,
    Lose
}

fn check_win(guess: u32, result: u32) -> GameResult {
    if result == 0 {
        return GameResult::Lose
    }

    if guess == result {
        return GameResult::Exact
    }

    if

    GameResult::Lose
}

fn bet(deps: DepsMut, env: Env, info: MessageInfo, guess: u32) -> Result<Response, CustomContractError> {

    let r: Binary = info.random;

    let mut prng = Prng::new(r.as_slice());

    // this is probably fine since the modulo bias is super small
    let x = prng.next_u32() % 37;



    Ok(Response::default())
}


fn read_share(deps: Deps, env: Env, user_index: u32) -> ReadShareResponse {
    // todo: authentication

    // read the shares from state

    return ReadShareResponse {
        user_share: "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".to_string(),
        chain_share: "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".to_string(),
        public_key: "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".to_string()
    }
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::ReadShare { user_index } => to_binary(&query_who_won(deps, env, game)?),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::msg::GameStateResponse;
    use crate::state::{GameResult, GameStatus, RPS};
    use cosmwasm_std::testing::{
        mock_dependencies, mock_env, mock_info, MockApi, MockQuerier, MockStorage,
    };
    use cosmwasm_std::{coins, OwnedDeps};

    fn instantiate_contract(deps: DepsMut) -> MessageInfo {
        let msg = InstantiateMsg {};
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps, mock_env(), info.clone(), msg).unwrap();
        info
    }

    #[test]
    fn new_game() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        let info = instantiate_contract(deps.as_mut());

        // let msg_player1 = ExecuteMsg::NewGame {
        //     player_name: "alice".to_string(),
        //     bet: None,
        // };
        //
        // // test new game returns a valid game ID
        // let res = execute(deps.as_mut(), mock_env(), info.clone(), msg_player1).unwrap();
        // assert_eq!(&res.events[0].attributes[0].value, "QTEBERJH");
        //
        // let game_id = res.events[0].attributes[0].value.clone();
        //
        // // it worked, let's query the state and check that we're waiting for the 2nd player to join
        // let unwrapped = _check_current_status(&deps, env, &game_id, GameStatus::WaitingForPlayerToJoin);
        // assert_eq!(unwrapped.game, game_id);
    }
}
