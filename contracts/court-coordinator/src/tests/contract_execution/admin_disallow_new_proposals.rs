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
		CourtExecuteMsg::Admin(CourtAdminExecuteMsg::AllowNewProposals { allowed: false }),
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
		CourtExecuteMsg::Admin(CourtAdminExecuteMsg::AllowNewProposals { allowed: false }),
	);
}

#[test]
pub fn cant_be_self() {
	let mut env_deps = new_env_and_instantiate(None);
	let contract_addr = &env_deps.0.contract.address.clone();
	helpers::execute_change_admin(&mut env_deps, None, contract_addr.as_str());

	let execute_response = helpers::execute(
		&mut env_deps,
		Some(MessageInfo {
			sender: contract_addr.clone(),
			funds: vec![],
		}),
		CourtExecuteMsg::Admin(CourtAdminExecuteMsg::AllowNewProposals { allowed: false }),
	);
	assert!(execute_response.is_err());
	assert!(execute_response.is_err_and(|err| {
		err.to_string()
			.contains("may result in this contract becoming unusable")
	}));
}

#[test]
pub fn blocks_proposals() {
	let mut env_deps = new_env_and_instantiate(Some(CourtInstantiateMsg {
		admin: Addr::unchecked(ADMIN_ACCOUNT),
		shares_mint_amount: 1000000u128.into(),
		shares_mint_receiver: Addr::unchecked(SHARES_HOLDER_ACCOUNT_2),
		minimum_vote_proposal_percent: 10,
		minimum_vote_turnout_percent: 10,
		minimum_vote_pass_percent: 50,
		max_proposal_expiry_time_seconds: 86400,
		execution_expiry_time_seconds: 86400,
		vote_share_name: "Test vote token".into(),
		vote_share_symbol: "TVT".into(),
		vote_share_description: "Test vote token".into(),
	}));
	// let vote_shares_denom = helpers::query_denom(&env_deps).unwrap().votes;
	helpers::execute_stake_votes(&mut env_deps, SHARES_HOLDER_ACCOUNT_2, 200000u128);
	helpers::execute_allow_new_proposals(&mut env_deps, None, false);
	let execute_response = helpers::execute(
		&mut env_deps,
		Some(MessageInfo {
			sender: Addr::unchecked(SHARES_HOLDER_ACCOUNT_2),
			funds: vec![],
		}),
		CourtExecuteMsg::ProposeTransaction {
			msgs: vec![ProposedCourtMsgJsonable::SendCoin {
				to: RANDOM_ACCOUNT_1.into(),
				denom: "usei".into(),
				amount: 31337u128.into(),
			}],
			expiry_time_seconds: 3600,
		},
	);
	assert!(
		execute_response.is_err_and(|err| { err.to_string().contains("proposals currently aren't being accepted") })
	);
	helpers::execute_allow_new_proposals(&mut env_deps, None, true);
	helpers::execute(
		&mut env_deps,
		Some(MessageInfo {
			sender: Addr::unchecked(SHARES_HOLDER_ACCOUNT_2),
			funds: vec![],
		}),
		CourtExecuteMsg::ProposeTransaction {
			msgs: vec![ProposedCourtMsgJsonable::SendCoin {
				to: RANDOM_ACCOUNT_1.into(),
				denom: "usei".into(),
				amount: 31337u128.into(),
			}],
			expiry_time_seconds: 3600,
		},
	)
	.unwrap();
}
