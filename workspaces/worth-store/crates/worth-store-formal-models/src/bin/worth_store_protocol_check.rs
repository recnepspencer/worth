#![cfg_attr(test, allow(clippy::disallowed_methods))]
use std::num::NonZeroU64;
use std::path::PathBuf;

use worth_store_formal_models::runner::{
    execute_protocol_check, ProtocolCheckBounds, ProtocolCheckInvocation, ProtocolCheckVerdict,
    TlcRunnerPaths,
};
use worth_store_formal_models::ProtocolFamily;

fn main() {
    let arguments = std::env::args().collect::<Vec<_>>();
    let [_, java, tool_jar, state_root] = arguments.as_slice() else {
        eprintln!("usage: worth-store-protocol-check <java> <tla2tools.jar> <state-root>");
        std::process::exit(2);
    };
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let state_root = PathBuf::from(state_root);
    let bounds = ProtocolCheckBounds::new(
        NonZeroU64::new(1_000_000).unwrap(),
        NonZeroU64::new(100).unwrap(),
    );

    for protocol in ProtocolFamily::all() {
        let invocation =
            ProtocolCheckInvocation::for_checked_protocol(protocol, &crate_root, bounds);
        let state_directory = state_root.join(format!("protocol-{protocol:?}").to_lowercase());
        let runner = TlcRunnerPaths::new(java, tool_jar, state_directory);
        match execute_protocol_check(&invocation, &runner) {
            Ok(ProtocolCheckVerdict::CheckedWithinBounds { statistics, .. }) => {
                println!(
                    "checked {protocol:?}: {} distinct states, depth {}",
                    statistics.distinct_states(),
                    statistics.trace_depth()
                );
            }
            Ok(verdict) => {
                eprintln!("protocol {protocol:?} did not check cleanly: {verdict:?}");
                std::process::exit(1);
            }
            Err(failure) => {
                eprintln!("protocol {protocol:?} runner failed: {failure:?}");
                std::process::exit(1);
            }
        }
    }
}
