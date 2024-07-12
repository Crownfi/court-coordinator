import { coin } from "@cosmjs/proto-signing";
import { applyEnvVarsToDefaultClientEnv, fundFromLocalKeychain } from "@crownfi/sei-cli-utils";
import { ContractDeployingClientEnv, UIAmount } from "@crownfi/sei-utils";
import { CourtInstantiateMsg, CourtCoordinatorContract } from "@crownfi/court-coordinator-sdk";
import * as wtfnode from "wtfnode";
import * as path from "path";
import {promises as fsp} from "fs";
const __dirname = import.meta.dirname;
console.log("Eyy");
applyEnvVarsToDefaultClientEnv();

const clientEnv = await ContractDeployingClientEnv.get();

console.log(`chainID: ${clientEnv.chainId} wallet: ${clientEnv.getAccount().seiAddress}`);
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
		admin: clientEnv.getAccount().seiAddress,
		max_proposal_expiry_time_seconds: 604800, // 1 week.
		execution_expiry_time_seconds: 259200, // 3 days.
		minimum_vote_pass_percent: 70,
		minimum_vote_proposal_percent: 10, 
		minimum_vote_turnout_percent: 50,
		shares_mint_receiver: clientEnv.getAccount().seiAddress,
		shares_mint_amount: "1000000000000",
		vote_share_name: "CrownFi Court",
		vote_share_symbol: "ccrt",
		vote_share_description: "CrownFi Court (DAO) voting shares"
	} satisfies CourtInstantiateMsg,
	"jewel_distributor"
);
console.log("Confirmed transaction", initResult.transactionHash);
console.log("New contract instantiated at", initResult.contractAddress);

let breaks = 5;
process.on("SIGINT", () => {
	wtfnode.dump();
	breaks -= 1;
	console.log(breaks, "break(s) left");
	if (breaks == 0) {
		process.exit(130);
	}
});
