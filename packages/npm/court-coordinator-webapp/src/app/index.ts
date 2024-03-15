import { ClientEnv, setDefaultNetwork } from "@crownfi/sei-utils";
import { q } from "@aritz-cracker/browser-utils";
import { setLoading } from "./loading.js";
import "./wallet_select/index.js";

export async function main() {
	let storedNetworkPref = localStorage.getItem("preferred_sei_network");
	if (storedNetworkPref == null) {
		storedNetworkPref = (
			document.location.host.startsWith("127.0.0.1") ||
			document.location.host.startsWith("localhost")
		) ? "sei-chain" : "atlantic-2";
	}
	setDefaultNetwork(storedNetworkPref);
	let storedProviderPref = localStorage.getItem("preferred_sei_provider");
	ClientEnv.setDefaultProvider(storedProviderPref as any, true);
	setLoading(false);

	const mainContent = q("#main-content") as HTMLElement;
	// TODO: Add elements
}
