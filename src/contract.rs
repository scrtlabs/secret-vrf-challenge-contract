use cosmwasm_std::{
    entry_point, to_binary, BankMsg, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo,
    QueryResponse, Response, StdError, StdResult, Uint128,
};

use base32;

use crate::errors::CustomContractError;
use crate::errors::CustomContractError::Std;
use crate::msg::{CheckWinner, ExecuteMsg, GameStateResponse, InstantiateMsg, QueryMsg};
use crate::rng::Prng;
use crate::state::{load_game_state, save_game_state, CurrentStatus, GameResult, Player, State, RPS, calculate_winner};

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
        ExecuteMsg::NewGame { bet, name } => try_new_game(deps, env, info, bet, name),
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
        QueryMsg::GameState { game } => to_binary(&query_game_state(deps, game)?),
    }
}

pub fn try_join(
    deps: DepsMut,
    info: MessageInfo,
    name: String,
    game: String,
) -> Result<Response, CustomContractError> {
    deps.api.debug(&format!("Player 2 - {:?} is joining the game", &info.sender));
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

    deps.api.debug(&format!("Done. Current players: {:?}", &state.players));

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

    if env.block.height < state.game_state.end_game_block.unwrap() {
        return Err(Std(StdError::generic_err("Cannot finalize right now")));
    }

    let mut messages = vec![];

    // handle giving out the money
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
            GameResult::Tie => {
            }
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
    deps.api.debug(&format!("Submitting choice {:?} for player {:?} ", &choice, &info.sender));
    let mut state = load_game_state(deps.storage, &game)?;
    deps.api.debug(&format!("Current players: {:?}", &state.players));

    match state.game_state.status {
        CurrentStatus::Started | CurrentStatus::Got1stChoice => {}
        _ => return Err(Std(StdError::generic_err("Cannot submit choice right now"))),
    }

    _set_choice_for_player(info, choice, &mut state)?;
    deps.api.debug(&format!("Done. Current choices: {:?}", &state.choices));
    state.next();

    if state.game_state.status == CurrentStatus::DoneGettingChoices {
        state.game_state.end_game_block = Some(env.block.height);

        if let (Some(choice1), Some(choice2)) = (&state.choices[0], &state.choices[1]) {
            state.game_state.winner = Some(calculate_winner(choice1, choice2));
            deps.api.debug(&format!("Winner is: {:?}", &state.game_state.winner));
        } else {
            panic!("Got choices but they weren't saved")
        }
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
            return Ok(());
        }
    }
    if let Some(player) = &state.players[1] {
        if &info.sender == player.address() {
            state.choices[1] = Some(choice);
            return Ok(());
        }
    }
    Err(Std(StdError::generic_err("Sender is not in this game")))

}

fn get_random_game_id(env: &Env, info: &MessageInfo) -> String {

    let mut seed_vec: Vec<u8> = vec![];

    seed_vec.extend_from_slice(&env.block.height.to_be_bytes());
    seed_vec.extend_from_slice(&info.sender.as_bytes());

    let entropy = if let Some(tx) = &env.transaction {
        tx.index.to_be_bytes()
    } else {
        [0u8; 4]
    };

    let mut rng = Prng::new(&env.block.height.to_be_bytes(), &entropy);

    let rand = rng.rand_bytes();
    let sub_slice = rand.split_at(5);
    // we use base32 with crockford alphabet to produce a more human-readable string
    return base32::encode(base32::Alphabet::Crockford, &sub_slice.0);
}

pub fn try_new_game(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    bet: Option<Coin>,
    name: String,
) -> Result<Response, CustomContractError> {
    let mut state = State::default();

    let game = get_random_game_id(&env, &info);

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
        cosmwasm_std::Event::new("new_rps_game".to_string()).add_attribute("game_code", game);

    Ok(resp.add_events(vec![new_evt]))
}


fn query_game_state(deps: Deps, game: String) -> StdResult<GameStateResponse> {
    let state = load_game_state(deps.storage, &game).map_err(
        |_| StdError::generic_err("Game with this ID not found")
    )?;

    return Ok(GameStateResponse { game, state: state.game_state.status })
}

fn query_who_won(deps: Deps, env: Env, game: String) -> StdResult<CheckWinner> {
    let state = load_game_state(deps.storage, &game)?;

    if state.game_state.status != CurrentStatus::Finalized {
        return Err(StdError::generic_err(
            "Players didn't finish submitting their choice!",
        ));
    }

    if state.game_state.end_game_block.unwrap() > env.block.height {
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

    use cosmwasm_std::{coins, OwnedDeps};
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info, MockApi, MockQuerier, MockStorage};

    #[test]
    fn proper_instantialization() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {};
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
    }

    #[test]
    fn new_game() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {};
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let msg_player1 = ExecuteMsg::NewGame {
            bet: None,
            name: "alice".to_string(),
        };
        let res = execute(deps.as_mut(), mock_env(), info.clone(), msg_player1).unwrap();
        assert_eq!(&res.events[0].attributes[0].value, "QTEBERJH");

        let game_id = res.events[0].attributes[0].value.clone();

        // it worked, let's query the state
        let unwrapped = _check_current_status(&deps, &game_id, CurrentStatus::WaitingForPlayerToJoin);
        assert_eq!(unwrapped.game, game_id);
    }

    fn _check_current_status(deps: &OwnedDeps<MockStorage, MockApi, MockQuerier>, game_id: &String, expected: CurrentStatus) -> GameStateResponse {
        let value = query_game_state(deps.as_ref(), game_id.clone());

        if value.is_err() {
            panic!("Game not found in storage");
        }

        let unwrapped = value.unwrap();

        assert_eq!(&unwrapped.state, &expected);
        unwrapped
    }

    #[test]
    fn full_game() {
        let mut deps = mock_dependencies();
        let mut env = mock_env();

        let msg = InstantiateMsg {};
        let info = mock_info("alice", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let msg_player1 = ExecuteMsg::NewGame {
            bet: None,
            name: "alice".to_string(),
        };
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg_player1).unwrap();
        assert_eq!(&res.events[0].attributes[0].value, "QTEBERJH");

        let game_id = res.events[0].attributes[0].value.clone();

        let msg_player2 = ExecuteMsg::JoinGame {
            name: "bob".to_string(),
            game: game_id.clone()
        };

        let info2 = mock_info("bob", &coins(2, "token"));
        let _res = execute(deps.as_mut(), env.clone(), info2.clone(), msg_player2).unwrap();

        let _ = _check_current_status(&deps, &game_id, CurrentStatus::Started);

        let msg_action_p1 = ExecuteMsg::SubmitChoice {
            game: game_id.clone(),
            choice: RPS::Rock
        };
        let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg_action_p1).unwrap();

        let _ = _check_current_status(&deps, &game_id, CurrentStatus::Got1stChoice);

        let msg_action_p2 = ExecuteMsg::SubmitChoice {
            game: game_id.clone(),
            choice: RPS::Paper
        };
        let _res = execute(deps.as_mut(), env.clone(), info2.clone(), msg_action_p2).unwrap();

        let _ = _check_current_status(&deps, &game_id, CurrentStatus::DoneGettingChoices);

        env.block.height += 1;

        let msg_finalize = ExecuteMsg::Finalize {
            game: game_id.clone(),
        };
        let _res = execute(deps.as_mut(), env.clone(), info2.clone(), msg_finalize).unwrap();



        let winner = query_who_won(deps.as_ref(), env, game_id);

        if winner.is_err() {
            panic!("Winner not available");
        }

        let unwrapped = winner.unwrap();

        assert_eq!(unwrapped.winner, GameResult::Player2);
        assert_eq!(unwrapped.address, Some(cosmwasm_std::Addr::unchecked("bob")));

    }

    //
    // #[test]
    // fn test_reset_state() {
    //     let mut deps = mock_dependencies();
    //
    //     let msg = InstantiateMsg {};
    //     let info = mock_info("creator", &coins(2, "token"));
    //     let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
    //
    //     let msg_player1 = ExecuteMsg::SubmitNetWorth {
    //         worth: 1,
    //         name: "alice".to_string(),
    //     };
    //
    //     let info = mock_info("creator", &[]);
    //     let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg_player1).unwrap();
    //
    //     let reset_msg = ExecuteMsg::Reset {};
    //     let _res = execute(deps.as_mut(), mock_env(), info.clone(), reset_msg).unwrap();
    //
    //     let msg_player2 = ExecuteMsg::SubmitNetWorth {
    //         worth: 2,
    //         name: "bob".to_string(),
    //     };
    //     let msg_player3 = ExecuteMsg::SubmitNetWorth {
    //         worth: 3,
    //         name: "carol".to_string(),
    //     };
    //
    //     let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg_player2).unwrap();
    //     let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg_player3).unwrap();
    //
    //     // it worked, let's query the state
    //     let value = query_who_won(deps.as_ref()).unwrap();
    //
    //     assert_eq!(&value.richer, "carol")
    // }
}
