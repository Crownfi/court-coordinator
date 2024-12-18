import { StakingInputsAutogen } from "./_autogen.js";
import { addUserTokenInfo, ClientEnv, getUserTokenInfo, hasUserTokenInfo, seiUtilEventEmitter, stringDecimalToBigInt } from "@crownfi/sei-utils";
import { applyCustomElementsWorkaround, disableFormInputs, enableFormInputs, qa } from "@aritz-cracker/browser-utils";
import { coin } from "@cosmjs/proto-signing";
import { getCourtCoordinatorFromChainId } from "@crownfi/court-coordinator-sdk";
import { FullscreenLoadingTask, msgBoxIfThrow } from "@crownfi/css-gothic-fantasy";
import { TokenDisplayElement, WebClientEnv } from "@crownfi/sei-webui-utils";
await applyCustomElementsWorkaround();

export class StakingInputsElement extends StakingInputsAutogen {
	constructor() {
		super();
		this.refs.formStake.addEventListener("submit", (ev) => {
			ev.preventDefault();
			if (!this.refs.formStake.elements.amount.value) {
				this.refs.formStake.elements.amount.value = "0";
				this.refs.formStake.elements.amount.required = true;
				this.refs.formStake.reportValidity();
				return;
			}
			msgBoxIfThrow(async () => {
				const task = new FullscreenLoadingTask();
				try{
					task.text = "Connecting to wallet...";
					task.show();
					const client = await WebClientEnv.get();
					const contract = getCourtCoordinatorFromChainId(client.queryClient, client.chainId);
					task.text = "Querying contract...";
					const {votes: voteSharesDenom} = await contract.queryDenom();
					const inAmount = stringDecimalToBigInt(
						this.refs.formStake.elements.amount.value,
						getUserTokenInfo(voteSharesDenom).decimals
					);
					if (inAmount == null) {
						return;
					}
					task.text = "";
					await client.executeContract(contract.buildStakeIx([coin(inAmount + "", voteSharesDenom)]))
				}finally{
					task.hide();
					this.refreshBalances();
				}
			});
		});
		this.refs.buttonUnstake.addEventListener("click", (ev) => {
			ev.preventDefault();
			msgBoxIfThrow(async () => {
				const task = new FullscreenLoadingTask();
				try{
					task.text = "Connecting to wallet...";
					task.show();
					const client = await WebClientEnv.get();
					task.text = "Querying contract...";
					const contract = getCourtCoordinatorFromChainId(client.queryClient, client.chainId);
					const ixs = [];
					if ((await contract.queryGetUserActiveProposals({
						descending: false,
						limit: 1,
						user: client.getAccount().seiAddress
					})).length) {
						ixs.push(contract.buildDeactivateVotesIx());
					}
					task.text = "";
					ixs.push(contract.buildUnstakeIx());
					await client.executeContractMulti(ixs);
				}finally{
					task.hide();
					this.refreshBalances();
				}
			});
		});
		
		this.refreshBalances();
	}
	connectedCallback() {
		this.classList.add("framed-box-small");
	}
	#shouldRefreshBalances: boolean = false;
	#isRefreshingBalances: boolean = false;
	refreshBalances() {
		this.#shouldRefreshBalances = true;
		if (this.#isRefreshingBalances) {
			return;
		}
		this.#isRefreshingBalances = true;
		disableFormInputs(this.refs.formStake);
		this.refs.buttonUnstake.disabled = true;
		(async () => {
			do {
				this.#shouldRefreshBalances = false;
				this.refs.stakedBalance.innerText = "";
				this.refs.stakedBalance.classList.add("loading-spinner-inline");
				this.refs.unstakedBalance.innerText = "";
				this.refs.unstakedBalance.classList.add("loading-spinner-inline");

				const client = await ClientEnv.get();
				if (client.account == null) {
					this.refs.stakedBalance.innerText = "[Not connected]";
					this.refs.unstakedBalance.innerText = "[Not connected]";
					return;
				}
				this.refs.buttonUnstake.disabled = false;
				const contract = getCourtCoordinatorFromChainId(client.queryClient, client.chainId);
				const {votes: voteSharesDenom} = await contract.queryDenom();
				if (!hasUserTokenInfo(voteSharesDenom)) {
					await addUserTokenInfo(client.queryClient, client.chainId, voteSharesDenom);
				}
				const minAmount = 10 ** -(getUserTokenInfo(voteSharesDenom).decimals);
				this.refs.formStake.elements.amount.step = minAmount + "";
				this.refs.formStake.elements.amount.min = minAmount + "";
				enableFormInputs(this.refs.formStake);
				await Promise.all([
					(async () => {
						const userStats = await contract.queryUserStats({user: client.getAccount().seiAddress});
						const tokenDisplay = new TokenDisplayElement();
						tokenDisplay.denom = voteSharesDenom;
						tokenDisplay.amount = userStats.staked_votes;
						this.refs.stakedBalance.replaceChildren(tokenDisplay);
						this.refs.stakedBalance.classList.remove("loading-spinner-inline");
					})(),
					(async () => {
						const userBalance = await client.getBalance(voteSharesDenom);
						const tokenDisplay = new TokenDisplayElement();
						tokenDisplay.denom = voteSharesDenom;
						tokenDisplay.amount = userBalance + "";
						this.refs.unstakedBalance.replaceChildren(tokenDisplay);
						this.refs.unstakedBalance.classList.remove("loading-spinner-inline");
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
			this.refs.stakedBalance.classList.remove("loading-spinner-inline");
			this.refs.unstakedBalance.classList.remove("loading-spinner-inline");
			this.#isRefreshingBalances = false;
		});
	}
}
StakingInputsElement.registerElement();
seiUtilEventEmitter.on("defaultNetworkChanged", (ev) => {
	(qa("[is=\"staking-inputs\"]") as NodeListOf<StakingInputsElement>).forEach(elem => {
		elem.refreshBalances();
	});
});

seiUtilEventEmitter.on("defaultProviderChanged", (ev) => {
	(qa("[is=\"staking-inputs\"]") as NodeListOf<StakingInputsElement>).forEach(elem => {
		elem.refreshBalances();
	});
});
