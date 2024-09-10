import { CourtCoordinatorContract, TransactionProposalExecutionStatus, TransactionProposalStatus } from "./base/index.js";
import { WasmExtension } from '@cosmjs/cosmwasm-stargate';
import { QueryClient } from "@cosmjs/stargate";
export * from "./base/index.js";

const knownContractAddresses: {[chainId: string]: string} = {
	"atlantic-2": "sei1ht5fsda2mlz6740jmpegctcrl4nmk685mlfgjv43fqxdph80f6tq5dh2t6"
};

export function getCourtCoordinatorAddressFromChainId(chainId: string): string {
	let contractAddress = knownContractAddresses[chainId];
	if (!contractAddress && typeof localStorage !== "undefined") {
		contractAddress = localStorage.getItem("@crownfi/court-coordinator-sdk/contract_address/" + chainId) || "";
	}
	if (!contractAddress && typeof window !== "undefined" && "prompt" in window) {
		const result = window.prompt("CourtCoordinatorContract address:");
		if (!result) {
			throw new Error("There's no default CourtCoordinator contract address for " + chainId);
		}
		contractAddress = result;
		localStorage.setItem("@crownfi/court-coordinator-sdk/contract_address/" + chainId, contractAddress);
	}
	if (!contractAddress) {
		throw new Error("There's no default CourtCoordinator contract address for " + chainId);
	}
	return contractAddress;
}

export function getCourtCoordinatorFromChainId<Q extends QueryClient & WasmExtension>(
	queryClient: Q, chainId: string
) {
	return new CourtCoordinatorContract(
		queryClient,
		getCourtCoordinatorAddressFromChainId(chainId)
	);
}

export function isProposalFinalized(proposalStatus: TransactionProposalStatus) {
	return proposalStatus == "executed" ||
		proposalStatus == "execution_expired" ||
		proposalStatus == "rejected" ||
		proposalStatus == "rejected_or_expired";
}
