use bpaf::Bpaf;
use court_coordinator_contract::msg::{CourtExecuteMsg, CourtInstantiateMsg, CourtQueryMsg};
use court_coordinator_contract::contract::{COURT_CONTRACT_NAME, COURT_CONTRACT_VERSION};
use crownfi_sei_sdk_autogen::CrownfiSdkMaker;
use std::path::PathBuf;
type Void = ();

#[derive(Clone, Debug, Bpaf)]
#[bpaf(options, version)]
struct MakeSdkOptions {
	#[bpaf(positional("OUT_DIR"))]
	/// The path to save the auto-generated typescript to
	out_dir: PathBuf,
}

fn main() -> color_eyre::Result<()> {
	color_eyre::install()?;
	let args = make_sdk_options().run();
	CrownfiSdkMaker::new()
		.add_contract_with_version::<
			CourtInstantiateMsg,
			CourtExecuteMsg,
			CourtQueryMsg,
			Void,
			Void,
			Void
		>(
			"court_coordinator",
			Some((COURT_CONTRACT_NAME.into(), COURT_CONTRACT_VERSION.into()))
		)?
		.generate_code(args.out_dir)?;
	Ok(())
}
