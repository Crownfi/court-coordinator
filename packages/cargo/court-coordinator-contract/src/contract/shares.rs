use cosmwasm_std::{Coin, Env};

pub static VOTES_SUBDENOM: &str = "votes";

#[inline]
pub fn votes_denom(env: &Env) -> String {
	// TODO: See if we can use OnceCell or something
	"factory/".to_string() + env.contract.address.as_str() + "/" + VOTES_SUBDENOM
}
#[inline]
pub fn votes_coin(env: &Env, amount: u128) -> Coin {
	Coin::new(amount, votes_denom(env))
}
