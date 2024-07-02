use super::*;

#[test]
pub fn correct_tokens_check() {
	let mut env_deps = new_env_and_instantiate(None);
	let contract_addr = &env_deps.0.contract.address.clone();
	let vote_shares_denom = format!("factory/{}/votes", contract_addr);

	helpers::assert_must_pay(
		&mut env_deps,
		SHARES_HOLDER_ACCOUNT_1,
		CourtExecuteMsg::Stake,
		&vote_shares_denom,
	);
}

#[test]
pub fn info_updated() {
	let mut env_deps = new_env_and_instantiate(None);
	let contract_addr = &env_deps.0.contract.address.clone();
	let vote_shares_denom = format!("factory/{}/votes", contract_addr);

	assert_eq!(
		helpers::query_user_stats(&mut env_deps, SHARES_HOLDER_ACCOUNT_1).unwrap(),
		CourtUserStatsJsonable {
			staked_votes: 0u128.into()
		}
	);

	let execute_response = helpers::execute(
		&mut env_deps,
		Some(MessageInfo {
			sender: Addr::unchecked(SHARES_HOLDER_ACCOUNT_1),
			funds: vec![coin(31337, &vote_shares_denom)],
		}),
		CourtExecuteMsg::Stake,
	)
	.unwrap();

	assert_eq!(
		helpers::query_user_stats(&mut env_deps, SHARES_HOLDER_ACCOUNT_1).unwrap(),
		CourtUserStatsJsonable {
			staked_votes: 31337u128.into()
		}
	);
	assert_eq!(
		execute_response.events,
		vec![cosmwasm_std::Event::new("stake")
			.add_attribute("user", SHARES_HOLDER_ACCOUNT_1)
			.add_attribute("user_new_votes", 31337.to_string())
			.add_attribute("user_total_votes", 31337.to_string())]
	);

	let execute_response = helpers::execute(
		&mut env_deps,
		Some(MessageInfo {
			sender: Addr::unchecked(SHARES_HOLDER_ACCOUNT_1),
			funds: vec![coin(8663, &vote_shares_denom)],
		}),
		CourtExecuteMsg::Stake,
	)
	.unwrap();

	assert_eq!(
		helpers::query_user_stats(&mut env_deps, SHARES_HOLDER_ACCOUNT_1).unwrap(),
		CourtUserStatsJsonable {
			staked_votes: 40000u128.into()
		}
	);
	assert_eq!(
		execute_response.events,
		vec![cosmwasm_std::Event::new("stake")
			.add_attribute("user", SHARES_HOLDER_ACCOUNT_1)
			.add_attribute("user_new_votes", 8663.to_string())
			.add_attribute("user_total_votes", 40000.to_string())]
	);
}
