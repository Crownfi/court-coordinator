import { addErrorMsgFormatter } from "@crownfi/css-gothic-fantasy";
import { TokenDisplayElement } from "@crownfi/sei-webui-utils";

export class NotEnoughStakedVotesForProposalError extends Error {
	name!: "NotEnoughStakedVotesForProposalError";
	required: bigint;
	actual: bigint;
	denom: string;
	constructor(required: bigint, actual: bigint, denom: string) {
		super(required + " voting shares are required to create a proposal and user has " + actual + "");
		this.denom = denom;
		this.required = required;
		this.actual = actual;
	}
}
NotEnoughStakedVotesForProposalError.prototype.name = "NotEnoughStakedVotesForProposalError";

addErrorMsgFormatter((err) => {
	if (!(err instanceof NotEnoughStakedVotesForProposalError)) {
		return;
	}
	const message = document.createElement("p");
	const requiredVoteDisplay = new TokenDisplayElement();
	requiredVoteDisplay.amount = err.required + "";
	requiredVoteDisplay.denom = err.denom;
	const actualVoteDisplay = new TokenDisplayElement();
	actualVoteDisplay.amount = err.actual + "";
	actualVoteDisplay.denom = err.denom;
	message.append(
		"You've staked ",
		actualVoteDisplay,
		", yet you must stake at least ",
		requiredVoteDisplay,
		" to create a proposal."
	);
	return {
		title: "Not enough staked votes",
		message,
		dialogClass: "warning",
		dialogIcon: "warning",
		hideDetails: true
	};
});
