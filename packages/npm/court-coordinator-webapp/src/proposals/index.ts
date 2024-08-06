import { alert, FullscreenLoadingTask, msgBoxIfThrow, prompt } from "@crownfi/css-gothic-fantasy";
import { CourtProposalsContainerAutogen } from "./_autogen.js";
import { ClientEnv, seiUtilEventEmitter } from "@crownfi/sei-utils";
import { getCourtCoordinatorFromChainId } from "@crownfi/court-coordinator-sdk";
import { NotEnoughStakedVotesForProposalError } from "../error.js";
import { qa } from "@aritz-cracker/browser-utils";
import { CourtProposalCreatorElement } from "./proposal_create.js";
import { CourtProposalElement } from "./proposal_view.js";

export * from "./proposal_view.js";
export * from "./proposal_create.js";

export class CourtProposalsContainerElement extends CourtProposalsContainerAutogen {
	constructor() {
		super();
		this.refs.newProposalButton.addEventListener("click", () => {
			msgBoxIfThrow(async () => {
				const task = new FullscreenLoadingTask();
				try {
					task.text = "Checking eligibility";
					task.show();
					const client = await ClientEnv.get();
					const contract = getCourtCoordinatorFromChainId(client.queryClient, client.chainId);
					const [totalSupply, contractConfig] = await Promise.all([
						contract.queryTotalSupply(),
						contract.queryConfig()
					]);
					const votesRequiredForProposal = BigInt(totalSupply.votes) * BigInt(
						contractConfig.minimum_vote_proposal_percent
					) / 100n;
					if (client.account == null) {
						throw new NotEnoughStakedVotesForProposalError(votesRequiredForProposal, 0n);
					}
					const userStats = await contract.queryUserStats({user: client.account.seiAddress});
					const userStakesVotes = BigInt(userStats.staked_votes);
					if (userStakesVotes < votesRequiredForProposal) {
						throw new NotEnoughStakedVotesForProposalError(votesRequiredForProposal, userStakesVotes);
					}
					CourtProposalCreatorElement.showModalAndDoTransaction();
				} finally {
					task.hide();
				}
			})
		});
		this.refs.proposalSelectButton.addEventListener("click", () => {
			msgBoxIfThrow(async () => {
				const proposalId = await prompt("View proposal", "Enter proposal ID", "", "search");
				if (!proposalId) {
					return;
				}
				this.querySelectorAll("[is=\"court-proposal-placeholder\"]").forEach(elem => {
					elem.remove();
				});
				this.querySelectorAll("[is=\"court-proposal\"]").forEach(elem => {
					elem.remove();
				});
				const newElement = new CourtProposalElement();
				newElement.proposalId = proposalId + "";
				this.append(newElement);
			});
		});
	}

	#isRefreshing: boolean = false;
	#shouldRefresh: boolean = false;
	refresh() {
		this.#shouldRefresh = true;
		if (this.#isRefreshing) {
			return;
		}
		this.#isRefreshing = true;
		this.refs.newProposalButton.disabled = true;
		this.refs.newProposalButton.classList.add("lazy-loading");
		msgBoxIfThrow(async () => {
			try {
				do {
					this.#shouldRefresh = false;
					const client = await ClientEnv.get();
					// const contract = getCourtCoordinatorFromChainId(client.queryClient, client.chainId);
					this.refs.newProposalButton.disabled = !client.hasAccount();
					this.refs.newProposalButton.classList.remove("lazy-loading");
				}while(this.#shouldRefresh);
			} finally {
				this.#isRefreshing = false;
			}
		});
	}
	connectedCallback() {
		this.refresh();
	}
}
CourtProposalsContainerElement.registerElement();
seiUtilEventEmitter.on("defaultNetworkChanged", (ev) => {
	(qa("[is=\"court-proposals-container\"]") as NodeListOf<CourtProposalsContainerElement>).forEach(elem => {
		elem.refresh();
	});
});
seiUtilEventEmitter.on("defaultProviderChanged", (ev) => {
	(qa("[is=\"court-proposals-container\"]") as NodeListOf<CourtProposalsContainerElement>).forEach(elem => {
		elem.refresh();
	});
});
