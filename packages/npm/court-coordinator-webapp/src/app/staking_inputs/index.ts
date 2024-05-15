import { StakingInputsAutogen } from "./_autogen.js";
import { ClientEnv, seiUtilEventEmitter } from "@crownfi/sei-utils";
import { qa } from "@aritz-cracker/browser-utils";
import { errorDialogIfRejected } from "../dialogs/error.js";
import { setLoading } from "../loading.js";
import { ClientEnvWithModals } from "../fullscreen_client_env.js";
import { coin } from "@cosmjs/proto-signing";
import { getCourtCoordinatorFromChainId } from "@crownfi/court-coordinator-sdk";

export class StakingInputsElement extends StakingInputsAutogen {
	constructor() {
		super();
		this.refs.formStake.addEventListener("submit", (ev) => {
			ev.preventDefault();
			const inAmount = this.refs.formStake.elements.amount.value;
			errorDialogIfRejected(async () => {
				try{
					setLoading(true, "Connecting to wallet...");
					const client = await ClientEnvWithModals.get();
					const contract = await getCourtCoordinatorFromChainId(client.wasmClient, client.chainId);
					const {votes: voteSharesDenom} = await contract.queryDenom();
					await client.executeContractFullscreen(contract.buildStakeIx([coin(inAmount, voteSharesDenom)]))
				}finally{
					setLoading(false);
				}
			});
		});
		this.refs.formUnstake.addEventListener("submit", (ev) => {
			ev.preventDefault();
			const inAmount = this.refs.formUnstake.elements.amount.value;
			errorDialogIfRejected(async () => {
				try{
					setLoading(true, "Connecting to wallet...");
					const client = await ClientEnvWithModals.get();
					const contract = await getCourtCoordinatorFromChainId(client.wasmClient, client.chainId);
					const {votes: voteSharesDenom} = await contract.queryDenom();
					await client.executeContractFullscreen(contract.buildUnstakeIx([coin(inAmount, voteSharesDenom)]))
				}finally{
					setLoading(false);
				}
			});
		});
		this.refreshBalances();
	}

	#shouldRefreshBalances: boolean = false;
	#isRefreshingBalances: boolean = false;
	refreshBalances() {
		this.#shouldRefreshBalances = true;
		if (this.#isRefreshingBalances) {
			return;
		}
		this.#isRefreshingBalances = true;
		(async () => {
			do {
				this.#shouldRefreshBalances = false;
				// stuff here
				this.refs.stakedBalance.innerText = "";
				this.refs.stakedBalance.classList.add("lazy-loading-text-4");
				this.refs.unstakedBalance.innerText = "";
				this.refs.unstakedBalance.classList.add("lazy-loading-text-4");

				const client = await ClientEnv.get();
				if (client.account == null) {
					this.refs.stakedBalance.innerText = "[Not connected]";
					this.refs.unstakedBalance.innerText = "[Not connected]";
					return;
				}
				const contract = await getCourtCoordinatorFromChainId(client.wasmClient, client.chainId);
				const {votes: voteSharesDenom} = await contract.queryDenom();
				await Promise.all([
					(async () => {
						const userStats = await contract.queryUserStats({user: client.getAccount().address});
						this.refs.stakedBalance.innerText = userStats.staked_votes;
						this.refs.stakedBalance.classList.remove("lazy-loading-text-4");
					})(),
					(async () => {
						const userBalance = await client.getBalance(voteSharesDenom);
						this.refs.unstakedBalance.innerText = userBalance + "";
						this.refs.unstakedBalance.classList.remove("lazy-loading-text-4");
					})()
				]);
			} while(this.#shouldRefreshBalances);
		})().catch(ex => {
			if (!this.refs.stakedBalance.innerText) {
				this.refs.stakedBalance.innerText = "[Error]";
			}
			if (!this.refs.unstakedBalance.innerText) {
				this.refs.unstakedBalance.innerText = "[Error]";
			}
			console.error(ex);
		}).finally(() => {
			this.refs.stakedBalance.classList.remove("lazy-loading-text-4");
			this.refs.unstakedBalance.classList.remove("lazy-loading-text-4");
			this.#isRefreshingBalances = false;
		});
	}
}
StakingInputsElement.registerElement();
seiUtilEventEmitter.on("transactionConfirmed", (ev) => {
	(qa("[is=\"staking-inputs\"]") as NodeListOf<StakingInputsElement>).forEach(elem => {
		elem.refreshBalances()
	});
});
seiUtilEventEmitter.on("defaultProviderChanged", (ev) => {
	(qa("[is=\"staking-inputs\"]") as NodeListOf<StakingInputsElement>).forEach(elem => {
		elem.refreshBalances()
	});
});
