use super::*;

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
		CourtExecuteMsg::Admin(CourtAdminExecuteMsg::ChangeAdmin {
			admin: Addr::unchecked(RANDOM_ACCOUNT_2),
		}),
	);
}

#[test]
pub fn cannot_be_self_while_voting_disabled() {
	let mut env_deps = new_env_and_instantiate(None);
	let contract_addr = &env_deps.0.contract.address.clone();
	helpers::execute_allow_new_proposals(&mut env_deps, None, false);
	let execute_response = helpers::execute(
		&mut env_deps,
		Some(MessageInfo {
			sender: Addr::unchecked(ADMIN_ACCOUNT),
			funds: vec![],
		}),
		CourtExecuteMsg::Admin(CourtAdminExecuteMsg::ChangeAdmin {
			admin: contract_addr.clone(),
		}),
	);
	assert!(execute_response.is_err());
	assert!(execute_response.is_err_and(|err| {
		err.to_string()
			.contains("may result in this contract becoming unusable")
	}));
}

#[test]
pub fn new_guy_can_do_things() {
	let mut env_deps = new_env_and_instantiate(None);
	helpers::execute_change_admin(&mut env_deps, None, RANDOM_ACCOUNT_1);
	helpers::assert_only_authorized_instruction(
		&mut env_deps,
		&[],
		&[
			ADMIN_ACCOUNT,
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
		&[RANDOM_ACCOUNT_1],
		CourtExecuteMsg::Admin(CourtAdminExecuteMsg::ChangeConfig {
			minimum_vote_proposal_percent: Some(69),
			minimum_vote_turnout_percent: Some(69),
			minimum_vote_pass_percent: Some(69),
			max_proposal_expiry_time_seconds: None,
			execution_expiry_time_seconds: None,
		}),
	);
}
