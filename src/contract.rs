use crate::errors::CustomContractError;
use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};

use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};

use crate::executes::{try_join, try_new_game, try_submit_choice};
use crate::queries::{query_game_state, query_who_won};

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
        ExecuteMsg::NewGame { player_name, bet } => try_new_game(deps, env, info, bet, player_name),
        ExecuteMsg::JoinGame {
            game_code,
            player_name,
        } => try_join(deps, info, player_name, game_code),
        ExecuteMsg::SubmitChoice { game_code, choice } => {
            try_submit_choice(deps, info, env, game_code, choice)
        }
    }
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::WhoWon { game } => to_binary(&query_who_won(deps, env, game)?),
        QueryMsg::GameState { game } => to_binary(&query_game_state(deps, env, game)?),
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

    fn _check_current_status(
        deps: &OwnedDeps<MockStorage, MockApi, MockQuerier>,
        game_id: &String,
        expected: GameStatus,
    ) -> GameStateResponse {
        let value = query_game_state(deps.as_ref(), env, game_id.clone());

        if value.is_err() {
            panic!("Game not found in storage");
        }

        let unwrapped = value.unwrap();

        assert_eq!(&unwrapped.state, &expected);
        unwrapped
    }

    fn instantiate_contract(deps: DepsMut) -> MessageInfo {
        let msg = InstantiateMsg {};
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps, mock_env(), info.clone(), msg).unwrap();
        info
    }

    #[test]
    fn new_game() {
        let mut deps = mock_dependencies();

        let info = instantiate_contract(deps.as_mut());

        let msg_player1 = ExecuteMsg::NewGame {
            player_name: "alice".to_string(),
            bet: None,
        };

        // test new game returns a valid game ID
        let res = execute(deps.as_mut(), mock_env(), info.clone(), msg_player1).unwrap();
        assert_eq!(&res.events[0].attributes[0].value, "QTEBERJH");

        let game_id = res.events[0].attributes[0].value.clone();

        // it worked, let's query the state and check that we're waiting for the 2nd player to join
        let unwrapped = _check_current_status(&deps, &game_id, GameStatus::WaitingForPlayerToJoin);
        assert_eq!(unwrapped.game, game_id);
    }

    #[test]
    fn full_game() {
        let mut deps = mock_dependencies();
        let mut env = mock_env();

        let info = instantiate_contract(deps.as_mut());

        let msg_player1 = ExecuteMsg::NewGame {
            bet: None,
            player_name: "alice".to_string(),
        };
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg_player1).unwrap();

        assert_eq!(&res.events[0].attributes[0].value, "QTEBERJH");

        let game_id = res.events[0].attributes[0].value.clone();

        let msg_player2 = ExecuteMsg::JoinGame {
            player_name: "bob".to_string(),
            game_code: game_id.clone(),
        };

        let info2 = mock_info("bob", &coins(2, "token"));
        let _res = execute(deps.as_mut(), env.clone(), info2.clone(), msg_player2).unwrap();

        let _ = _check_current_status(&deps, &game_id, GameStatus::Started);

        let msg_action_p1 = ExecuteMsg::SubmitChoice {
            game_code: game_id.clone(),
            choice: RPS::Rock,
        };
        let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg_action_p1).unwrap();

        let _ = _check_current_status(&deps, &game_id, GameStatus::Got1stChoiceWaitingFor2nd);

        let msg_action_p2 = ExecuteMsg::SubmitChoice {
            game_code: game_id.clone(),
            choice: RPS::Paper,
        };
        let _res = execute(deps.as_mut(), env.clone(), info2.clone(), msg_action_p2).unwrap();

        let _ = _check_current_status(&deps, &game_id, GameStatus::Done);

        env.block.height += 1;

        let winner = query_who_won(deps.as_ref(), env, game_id);

        if winner.is_err() {
            panic!("Winner not available");
        }

        let unwrapped = winner.unwrap();

        assert_eq!(unwrapped.winner, GameResult::Player2);
        assert_eq!(
            unwrapped.address,
            Some(cosmwasm_std::Addr::unchecked("bob"))
        );
    }
}
