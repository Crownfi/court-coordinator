import { CosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { coin } from "@cosmjs/proto-signing";
import { applyEnvVarsToDefaultClientEnv, fundFromLocalKeychain } from "@crownfi/sei-cli-utils";
import { ContractDeployingClientEnv, UIAmount } from "@crownfi/sei-utils";
import { CourtInstantiateMsg, CourtCoordinatorContract } from "@crownfi/court-coordinator-sdk";
import * as wtfnode from "wtfnode";
import * as path from "path";
import {promises as fsp} from "fs";
const __dirname = import.meta.dirname;
const activeClients: CosmWasmClient[] = [];

async function main() {
	applyEnvVarsToDefaultClientEnv();
	const clientEnv = await ContractDeployingClientEnv.get();
	activeClients.push(clientEnv.wasmClient);
	console.log(`chainID: ${clientEnv.chainId} wallet: ${clientEnv.getAccount().address}`);
	console.log("Account balance:", UIAmount(await clientEnv.getBalance("usei"), "usei", true));
	if (clientEnv.chainId == "sei-chain" && (await clientEnv.getBalance("usei")) < 1000000n) {
		console.log("Funding account form \"admin\" in local keyring");
		await fundFromLocalKeychain("admin", clientEnv, coin(100000000, "usei"));
		await fundFromLocalKeychain("admin", clientEnv, coin(100000000, "uusdc"));
		await fundFromLocalKeychain("admin", clientEnv, coin(100000000, "uatom"));
	}
	const contractBinary = await fsp.readFile(
		path.resolve(__dirname, "..", "artifacts", "court_coordinator_contract.wasm")
	);
	console.log("Upload code...");
	const uploadResult = await clientEnv.uploadContract(contractBinary, false);
	console.log("Confirmed transaction", uploadResult.transactionHash);
	console.log("Uploaded binary with Code ID", uploadResult.codeId);

	console.log("Init contract...")
	const initResult = await clientEnv.instantiateContract(
		uploadResult.codeId,
		{
			admin: clientEnv.getAccount().address,
			max_expiry_time_seconds: 604800, // 1 week.
			minimum_vote_pass_percent: 70,
			minimum_vote_proposal_percent: 10, 
			minimum_vote_turnout_percent: 50,
			shares_mint_receiver: clientEnv.getAccount().address,
			shares_mint_amount: "1000000000000"
		} satisfies CourtInstantiateMsg,
		"jewel_distributor"
	);
	console.log("Confirmed transaction", initResult.transactionHash);
	console.log("New contract instantiated at", initResult.contractAddress);
	
	/*
	if (clientEnv.chainId == "sei-chain") {
		const contract = new JewelDistributorContract(clientEnv.wasmClient, initResult.contractAddress);
		
		const execResult = await clientEnv.executeContract(
			contract.buildUpdateShareProfitsIx(
				{asset_ids: []},
				[coin(1000000, "usei"), coin(1000000, "uusdc"), coin(1000000, "uatom")]
			)
		);
	}
	*/

}


(async () => {
	try {
		await main();
	}catch(ex: any) {
		console.error(ex);
		process.exitCode = 1;
	}
	activeClients.forEach(v => v.disconnect());
	activeClients.length = 0;
})();
let breaks = 5;
process.on("SIGINT", () => {
	wtfnode.dump();
	breaks -= 1;
	console.log(breaks, "break(s) left");
	if (breaks == 0) {
		process.exit(130);
	}
});
