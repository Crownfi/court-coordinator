import { DeliverTxResponse, ExecuteInstruction } from "@cosmjs/cosmwasm-stargate";
import { StdFee } from "@cosmjs/stargate";
import { ClientEnv, seiUtilEventEmitter } from "@crownfi/sei-utils";
import { setLoading } from "./loading.js";
import { transactionConfirmedDialog } from "./dialogs/tx_confirm.js";

export class ClientEnvWithModals extends ClientEnv {
	async executeContractMultiFullscreen(
		instructions: ExecuteInstruction[],
		memo?: string,
		fee?: "auto" | StdFee
	): Promise<DeliverTxResponse> {
		try{
			setLoading(true, "Waiting for transaction approval...");
			const transactionHash = await this.executeContractMulti(instructions, memo, fee, "broadcasted");
			setLoading(true, "Waiting for " + transactionHash + " to confirm...");
			const result = await this.waitForTxConfirm(transactionHash, 120000, true);
			seiUtilEventEmitter.emit("transactionConfirmed", {chainId: this.chainId, sender: this.getAccount().address, result});
			//transactionConfirmedDialog(this.chainId, transactionHash);
			return result;
		}finally{
			setLoading(false);
		}
	}
	executeContractFullscreen(
		instruction: ExecuteInstruction,
		memo?: string,
		fee?: "auto" | StdFee
	): Promise<DeliverTxResponse> {
		return this.executeContractMultiFullscreen([instruction], memo, fee);
	}
}
