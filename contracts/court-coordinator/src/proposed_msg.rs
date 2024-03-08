use borsh::{BorshDeserialize, BorshSerialize};
use cosmwasm_schema::schemars::{self, JsonSchema};
use cosmwasm_std::{Addr, Api, Binary, Coin, CosmosMsg, StdError, Uint128, WasmMsg};
use crownfi_cw_common::data_types::{asset::FungibleAssetKindString, canonical_addr::SeiCanonicalAddr};
use sei_cosmwasm::SeiMsg;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, BorshDeserialize, BorshSerialize)]
pub struct BorshableCoin {
	pub denom: String,
	pub amount: u128
}
impl From<Coin> for BorshableCoin {
	fn from(value: Coin) -> Self {
		Self {
			denom: value.denom,
			amount: value.amount.into()
		}
	}
}
impl From<BorshableCoin> for Coin {
	fn from(value: BorshableCoin) -> Self {
		Self {
			denom: value.denom,
			amount: value.amount.into()
		}
	}
}

#[derive(Clone, Debug, PartialEq, Eq, BorshDeserialize, BorshSerialize)]
pub enum ProposedCourtMsg {
	/// Sends a coin. Native or contract-driven
	SendCoin {
		to: SeiCanonicalAddr,
		denom: FungibleAssetKindString,
		amount: u128
	},
	ExecuteWasmContract {
		contract: SeiCanonicalAddr,
		msg: Vec<u8>,
		funds: Vec<BorshableCoin>
	},
	UpgradeWasmContract {
		contract: SeiCanonicalAddr,
		new_code_id: u64,
		msg: Vec<u8>,
	},
	ChangeWasmContractAdmin {
		contract: SeiCanonicalAddr,
		new_admin: SeiCanonicalAddr
	},
	ClearWasmContractAdmin {
		contract: SeiCanonicalAddr,
	},
	TokenfactoryMint {
		tokens: BorshableCoin
	}
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ProposedCourtMsgJsonable {
	/// Sends a coin
	SendCoin {
		to: Addr,
		denom: FungibleAssetKindString,
		amount: Uint128
	},
	ExecuteWasmContract {
		contract: Addr,
		msg: Binary,
		funds: Vec<Coin>
	},
	UpgradeWasmContract {
		contract: Addr,
		new_code_id: u64,
		msg: Binary,
	},
	ChangeWasmContractAdmin {
		contract: Addr,
		new_admin: Addr
	},
	ClearWasmContractAdmin {
		contract: Addr,
	},
	TokenfactoryMint {
		tokens: Coin
	}
}

impl ProposedCourtMsg {
	pub fn into_cosm_msg(self, api: &dyn Api) -> Result<CosmosMsg<SeiMsg>, StdError> {
		match self {
			ProposedCourtMsg::SendCoin { to, denom, amount } => {
				Ok(
					denom.into_asset(amount).transfer_to_msg(
						&to.into_addr_using_api(api)?
					)
				)
			},
			ProposedCourtMsg::ExecuteWasmContract { contract, msg, funds } => {
				Ok(
					WasmMsg::Execute {
						contract_addr: contract.into_addr_using_api(api)?.into_string(),
						msg: msg.into(),
						funds: funds.into_iter().map(|v| {v.into()}).collect()
					}.into()
				)
			},
			ProposedCourtMsg::UpgradeWasmContract { contract, new_code_id, msg } => {
				Ok(
					WasmMsg::Migrate {
						contract_addr: contract.into_addr_using_api(api)?.into_string(),
						new_code_id,
						msg: msg.into()
					}.into()
				)
			},
			ProposedCourtMsg::ChangeWasmContractAdmin { contract, new_admin } => {
				Ok(
					WasmMsg::UpdateAdmin {
						contract_addr: contract.into_addr_using_api(api)?.into_string(),
						admin: new_admin.into_addr_using_api(api)?.into_string()
					}.into()
				)
			},
			ProposedCourtMsg::ClearWasmContractAdmin { contract } => {
				Ok(
					WasmMsg::ClearAdmin {
						contract_addr: contract.into_addr_using_api(api)?.into_string()
					}.into()
				)
			},
			ProposedCourtMsg::TokenfactoryMint { tokens } => {
				Ok(
					SeiMsg::MintTokens { amount: tokens.into() }.into()
				)
			},
		}
	}
	pub fn into_jsonable(self, api: &dyn Api) -> Result<ProposedCourtMsgJsonable, StdError> {
		Ok(
			match self {
				ProposedCourtMsg::SendCoin { to, denom, amount } => {
					ProposedCourtMsgJsonable::SendCoin {
						to: to.into_addr_using_api(api)?,
						denom,
						amount: amount.into()
					}
				},
				ProposedCourtMsg::ExecuteWasmContract {contract, msg, funds } => {
					ProposedCourtMsgJsonable::ExecuteWasmContract {
						contract: contract.into_addr_using_api(api)?,
						msg: msg.into(),
						funds: funds.into_iter().map(|v| {v.into()}).collect()
					}
				},
				ProposedCourtMsg::UpgradeWasmContract { contract, new_code_id, msg } => {
					ProposedCourtMsgJsonable::UpgradeWasmContract {
						contract: contract.into_addr_using_api(api)?,
						new_code_id,
						msg: msg.into()
					}
				},
				ProposedCourtMsg::ChangeWasmContractAdmin { contract, new_admin } => {
					ProposedCourtMsgJsonable::ChangeWasmContractAdmin {
						contract: contract.into_addr_using_api(api)?,
						new_admin: new_admin.into_addr_using_api(api)?
					}
				},
				ProposedCourtMsg::ClearWasmContractAdmin { contract } => {
					ProposedCourtMsgJsonable::ClearWasmContractAdmin {
						contract: contract.into_addr_using_api(api)?
					}
				},
				ProposedCourtMsg::TokenfactoryMint { tokens } => {
					ProposedCourtMsgJsonable::TokenfactoryMint { tokens: tokens.into() }
				},
			}
		)
	}
}

impl ProposedCourtMsgJsonable {
	pub fn into_storable(self, api: &dyn Api) -> Result<ProposedCourtMsg, StdError> {
		Ok(
			match self {
				ProposedCourtMsgJsonable::SendCoin { to, denom, amount } => {
					ProposedCourtMsg::SendCoin {
						to: SeiCanonicalAddr::from_addr_using_api(&to, api)?,
						denom,
						amount: amount.into()
					}
				},
				ProposedCourtMsgJsonable::ExecuteWasmContract { contract, msg, funds } => {
					ProposedCourtMsg::ExecuteWasmContract {
						contract: SeiCanonicalAddr::from_addr_using_api(&contract, api)?,
						msg: msg.0,
						funds: funds.into_iter().map(|v| {v.into()}).collect()
					}
				},
				ProposedCourtMsgJsonable::UpgradeWasmContract { contract, new_code_id, msg } => {
					ProposedCourtMsg::UpgradeWasmContract {
						contract: SeiCanonicalAddr::from_addr_using_api(&contract, api)?,
						new_code_id,
						msg: msg.0
					}
				},
				ProposedCourtMsgJsonable::ChangeWasmContractAdmin { contract, new_admin } => {
					ProposedCourtMsg::ChangeWasmContractAdmin {
						contract: SeiCanonicalAddr::from_addr_using_api(&contract, api)?,
						new_admin: SeiCanonicalAddr::from_addr_using_api(&new_admin, api)?
					}
				},
				ProposedCourtMsgJsonable::ClearWasmContractAdmin { contract } => {
					ProposedCourtMsg::ClearWasmContractAdmin {
						contract: SeiCanonicalAddr::from_addr_using_api(&contract, api)?
					}
				},
				ProposedCourtMsgJsonable::TokenfactoryMint { tokens } => {
					ProposedCourtMsg::TokenfactoryMint { tokens: tokens.into() }
				},
			}
		)
	}
}
