//! Seals a co-committed aftermath fact into its commit's evidence at publication.
//!
//! Evidence outlives its commit's history, so the exact committed relation is
//! read once, at the published basis, and replay answers from the sealed copy
//! instead of a Relational read that history retirement may since have made
//! unreachable.

use super::stops::{failure, snapshot_capacity_failure};
use super::WorthQueryPendingApplicationPublication;
use crate::domain_computation::primary_graph::provider::WorthQueryPrimaryGraphProvider;
use crate::domain_computation::primary_graph::WorthQueryAftermathCausalityReadDenial;
use crate::domain_computation::WorthQueryProviderSessionFailure;

pub(super) fn seal(
    provider: &WorthQueryPrimaryGraphProvider,
    runtime: &mut worth_relational::facade::runtime::RelationalRuntime,
    pending: &mut WorthQueryPendingApplicationPublication,
    after: &worth_relational::facade::snapshots::SnapshotHandle,
) -> Result<(), WorthQueryProviderSessionFailure> {
    let Some(relation) = pending
        .attempt
        .as_ref()
        .and_then(|attempt| attempt.aftermath_causality())
        .cloned()
    else {
        return Ok(());
    };
    let application = pending
        .application
        .as_ref()
        .expect("pending application retains its evidence until cutover");
    if application.aftermath_causality().is_some() {
        return Ok(());
    }
    let committed = provider
        .resolve_aftermath_causality_in_snapshot(
            runtime,
            &pending.next_basis,
            after,
            &relation,
            Some(pending.outcome_identity),
        )
        .map_err(|denial| match denial {
            WorthQueryAftermathCausalityReadDenial::ActiveSnapshotCapacityExhausted { .. } => {
                snapshot_capacity_failure("aftermath causality could not be read at publication")
            }
            _ => failure("aftermath causality could not be read at publication"),
        })?
        .filter(|committed| committed.child() == application.commit_reference())
        .ok_or_else(|| failure("published commit does not carry its aftermath causality"))?;
    pending
        .application
        .as_mut()
        .expect("pending application retains its evidence until cutover")
        .seal_aftermath_causality(committed);
    Ok(())
}
