//! Provider-protocol result encoding after required publication is complete.

use super::WorthQueryPublishedApplicationCommit;
use crate::domain_computation::primary_graph::provider::session_commit::provider_failure;
use crate::domain_computation::primary_graph::provider::WorthQueryPrimaryGraphProvider;
use crate::domain_computation::WorthQueryProviderSessionFailure;
use crate::domain_computation::WorthQueryProviderSessionProtocolStage;

pub(super) fn encode(
    _provider: &WorthQueryPrimaryGraphProvider,
    _published: WorthQueryPublishedApplicationCommit,
) -> Result<
    crate::domain_computation::WorthQueryProviderTerminalDescription,
    WorthQueryProviderSessionFailure,
> {
    if _provider.take_lost_commit_response() {
        return Err(failure(
            "application commit response was lost after authoritative publication",
        ));
    }
    Ok(
        crate::domain_computation::WorthQueryProviderTerminalDescription::new(
            "primary application commit completed",
        )
        .expect("static provider terminal description is valid"),
    )
}

fn failure(detail: &'static str) -> WorthQueryProviderSessionFailure {
    provider_failure(WorthQueryProviderSessionProtocolStage::Commit, detail)
}
