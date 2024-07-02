use super::*;

#[test]
pub fn unfunded_check() {
	let mut env_deps = new_env_and_instantiate(None);

	helpers::assert_unfunded_instruction(
		&mut env_deps,
		Some(MessageInfo {
			sender: Addr::unchecked(ADMIN_ACCOUNT),
			funds: vec![],
		}),
		CourtExecuteMsg::Admin(CourtAdminExecuteMsg::MintShares {
			receiver: Addr::unchecked(RANDOM_ACCOUNT_1),
			amount: 31337u128.into(),
		}),
	);
}

#[test]
pub fn authorized_check() {
	let mut env_deps = new_env_and_instantiate(None);

	helpers::assert_only_authorized_instruction(
		&mut env_deps,
		&[],
		&[
			RANDOM_ACCOUNT_1,
			RANDOM_ACCOUNT_2,
			RANDOM_ACCOUNT_3,
			RANDOM_ACCOUNT_4,
			RANDOM_ACCOUNT_5,
			SHARES_HOLDER_ACCOUNT_1,
			SHARES_HOLDER_ACCOUNT_2,
			SHARES_HOLDER_ACCOUNT_3,
			SHARES_HOLDER_ACCOUNT_4,
			SHARES_HOLDER_ACCOUNT_5,
		],
		&[ADMIN_ACCOUNT],
		CourtExecuteMsg::Admin(CourtAdminExecuteMsg::MintShares {
			receiver: Addr::unchecked(RANDOM_ACCOUNT_1),
			amount: 31337u128.into(),
		}),
	);
}

#[test]
pub fn minted_check() {
	let mut env_deps = new_env_and_instantiate(None);
	// Just a sanity check since that's what new_env_and_instantiate does by default
	assert_eq!(get_known_vote_supply(&env_deps), 1000000u128);
	let vote_shares_denom = helpers::query_denom(&env_deps).unwrap().votes;

	let execute_reponse = helpers::execute(
		&mut env_deps,
		Some(MessageInfo {
			sender: Addr::unchecked(ADMIN_ACCOUNT),
			funds: vec![],
		}),
		CourtExecuteMsg::Admin(CourtAdminExecuteMsg::MintShares {
			receiver: Addr::unchecked(RANDOM_ACCOUNT_2),
			amount: 31337u128.into(),
		}),
	)
	.unwrap();

	// Mints the amount specified
	assert!(execute_reponse.messages.iter().any(|sub_msg| {
		match &sub_msg.msg {
			cosmwasm_std::CosmosMsg::Custom(sub_msg) => match sub_msg {
				sei_cosmwasm::SeiMsg::MintTokens { amount } => *amount == coin(31337u128, &vote_shares_denom),
				_ => false,
			},
			_ => false,
		}
	}));

	// Sends the amount of tokens minted to the user specified
	assert!(execute_reponse.messages.iter().any(|sub_msg| {
		match &sub_msg.msg {
			cosmwasm_std::CosmosMsg::Bank(sub_msg) => {
				*sub_msg
					== cosmwasm_std::BankMsg::Send {
						to_address: RANDOM_ACCOUNT_2.into(),
						amount: vec![coin(31337u128, &vote_shares_denom)],
					}
			}
			_ => false,
		}
	}));
	assert_eq!(get_known_vote_supply(&env_deps), 1031337u128);
}
