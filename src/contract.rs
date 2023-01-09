use crate::errors::CustomContractError;
use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};

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
        ExecuteMsg::CreateShare { public_key, shares, user_index } =>
            sss(deps, env, public_key, shares, user_index)
    }
}

fn sss(deps: DepsMut, env: Env, public_key: String, shares: Vec<String>, user_index: u32) -> Result<Response, CustomContractError> {

    // convert public key to EC Point
    // convert each of the shares to EC Scalar (?)

    // generate new secp256k1 private/public

    // compute <public user> + <public contract>

    // split <contract private key> into a bunch of pieces using shamir secret sharing

    // for each party:

    // // save a share of the split private key and the input shares for each of the users?

    // save the generated shares and input shares in state


    // example of secret sharing that doesn't work in wasm:

    // use vsss_rs::Shamir;
    // use k256::elliptic_curve::PrimeField;
    // use k256::{NonZeroScalar, Scalar, SecretKey};
    // // use rand::rngs::OsRng;
    //
    // let mut osrng = Prng::new(b"lol", b"lol");
    // let sk = SecretKey::random(&mut osrng);
    // let nzs = sk.to_nonzero_scalar();
    // let res = Shamir::<2, 3>::split_secret::<Scalar, Prng, 33>(*nzs.as_ref(), &mut osrng);
    // assert!(res.is_ok());
    // let shares = res.unwrap();
    // let res = Shamir::<2, 3>::combine_shares::<Scalar, 33>(&shares);
    // assert!(res.is_ok());
    // let scalar = res.unwrap();
    // let nzs_dup = NonZeroScalar::from_repr(scalar.to_repr()).unwrap();
    // let sk_dup = SecretKey::from(nzs_dup);
    // assert_eq!(sk_dup.to_be_bytes(), sk.to_be_bytes());

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
