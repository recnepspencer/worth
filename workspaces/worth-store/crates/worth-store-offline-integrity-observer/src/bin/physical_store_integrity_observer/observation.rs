use worth_store_offline_integrity_observer::{
    observe_store, OfflineIntegrityObservationRequest, OfflineIntegrityProtocolContext,
};

use super::arguments::ObserveArguments;

pub(super) fn observe(arguments: ObserveArguments) -> Result<(), String> {
    let process = std::process::id().to_string();
    let context = OfflineIntegrityProtocolContext::new(
        "physical_store_integrity_observer",
        process.clone(),
        arguments.run_identity.clone(),
        arguments.scenario_identity.clone(),
    )
    .map_err(|denial| format!("protocol context denied: {denial:?}"))?;
    let request = OfflineIntegrityObservationRequest::new(
        arguments.store_root,
        arguments.limits,
        arguments.report_destination,
        context,
    )
    .map_err(|denial| format!("observation request denied: {denial:?}"))?;
    let report =
        observe_store(&request).map_err(|denial| format!("observation denied: {denial:?}"))?;
    super::report_output::emit(&request, &report)
}
