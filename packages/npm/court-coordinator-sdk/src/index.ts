import { CosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { CourtCoordinatorContract, TransactionProposalExecutionStatus, TransactionProposalStatus } from "./base/index.js";
export * from "./base/index.js";

let localContractAddr = "";
let testContractAddr = "";

export async function getCourtCoordinatorFromChainId(
	endpoint: CosmWasmClient, chainId?: string
) {
	if (!chainId) {
		chainId = await endpoint.getChainId();
	}
	switch(chainId) {
		case "sei-chain":
			if (localContractAddr) {
				return new CourtCoordinatorContract(
					endpoint,
					localContractAddr
				);
			}
			if (typeof window !== "undefined" && "prompt" in window) {
				const result = window.prompt("CourtCoordinatorContract address:");
				if (!result) {
					throw new Error("There's no default CourtCoordinator contract address for " + chainId);
				}
				localContractAddr = result;
				return new CourtCoordinatorContract(
					endpoint,
					result
				);
			}
		case "atlantic-2":
			if (localContractAddr) {
				return new CourtCoordinatorContract(
					endpoint,
					localContractAddr
				);
			}
			if (typeof window !== "undefined" && "prompt" in window) {
				const result = window.prompt("CourtCoordinatorContract address:");
				if (!result) {
					throw new Error("There's no default CourtCoordinator contract address for " + chainId);
				}
				localContractAddr = result;
				return new CourtCoordinatorContract(
					endpoint,
					result
				);
			}
		default:
			throw new Error("There's no default CourtCoordinator contract address for " + chainId)
	}
}

export function isProposalFinalized(proposalStatus: TransactionProposalStatus | TransactionProposalExecutionStatus) {
	return proposalStatus == "executed" || proposalStatus == "cancelled";
}
