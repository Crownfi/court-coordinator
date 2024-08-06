import "@crownfi/sei-webui-utils";
import "dropdown-menu-element";
import { q } from "@aritz-cracker/browser-utils";
// import { RewardsTableElement } from "./rewards_table/index.js";
import { StakingInputsElement } from "./staking_inputs/index.js";
import { FantasyTabsElement, registerUnhandeledExceptionReporter } from "@crownfi/css-gothic-fantasy";
import { CourtConfigElement } from "./config_view/index.js";
import { CourtProposalsContainerElement } from "./proposals/index.js";
import "./timer_text/index.js";

const DOM_CONTENT_LOADED: Promise<void> = document.readyState == "loading" ? new Promise(resolve => {
	document.addEventListener("DOMContentLoaded", (_) => {
		resolve();
	})
}) : Promise.resolve();
const SEI_NETWORK_CONNECTED: Promise<void> = new Promise(resolve => {
	document.addEventListener("initialSeiConnection", (_) => {
		resolve();
	}, {once: true});
});

registerUnhandeledExceptionReporter(
	(window.location.host.startsWith("127.") || window.location.host.startsWith("localhost")) ? 2000 : 10000
);

export async function main() {
	await DOM_CONTENT_LOADED;
	await SEI_NETWORK_CONNECTED;
	const mainTabs = q("#main-tabs") as FantasyTabsElement;
	const mainContent = q("#main-content") as HTMLElement;
	mainTabs.addEventListener("fantasyTabSelected", (ev) => {
		mainContent.innerHTML = "";
		switch (ev.detail.value) {
			case "user":
				mainContent.appendChild(
					new StakingInputsElement()
				);
				break;
			case "dao":
				mainContent.appendChild(
					new CourtProposalsContainerElement()
				);
				break;
			case "config":
				mainContent.appendChild(
					new CourtConfigElement()
				);
				break;
			default:
				// No default
		}
	});
	/*
	mainContent.appendChild(
		new RewardsTableElement()
	);
	*/
	/*
	mainContent.appendChild(
		new StakingInputsElement()
	);
	*/
	
};
await main();
