use cosmwasm_std::{Env, MessageInfo};
use secret_toolkit_crypto::Prng;

pub fn get_random_game_id(env: &Env, info: &MessageInfo) -> String {

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