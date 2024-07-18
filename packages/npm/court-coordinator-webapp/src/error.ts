import { addErrorMsgFormatter } from "@crownfi/css-gothic-fantasy";

export class NotEnoughStakedVotesForProposalError extends Error {
	name!: "NotEnoughStakedVotesForProposalError";
	required: bigint;
	actual: bigint;
	constructor(required: bigint, actual: bigint) {
		super(required + " voting shares are required to create a proposal and user has " + actual + "");
		this.required = required;
		this.actual = actual;
	}
}
NotEnoughStakedVotesForProposalError.prototype.name = "NotEnoughStakedVotesForProposalError";

addErrorMsgFormatter((err) => {
	if (!(err instanceof NotEnoughStakedVotesForProposalError)) {
		return;
	}
	return {
		title: "Not enough staked votes",
		message: "You must stake at least " + err.required + " voting shares to create a proposal.\n" +
			"However, you've staked " + err.actual + " voting shares.",
		dialogClass: "warning",
		dialogIcon: "warning",
		hideDetails: true
	};
});
