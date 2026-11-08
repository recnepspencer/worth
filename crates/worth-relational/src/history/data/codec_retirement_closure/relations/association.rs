//! Cold association authenticates owned succession before independent reference classification.
use super::{charge, CodecRetirementClosureStop as Stop};
use crate::branch::RelationalBranchRoot;
use crate::history::data::canonical_commit_envelope::CanonicalRequiredOutputOrigin;
use std::sync::Arc;
use worth_execution::{ExecutionResourceLease, MapKernelContext};

pub(in super::super) fn validate_complete_publication(
    runtime: &crate::history::data::codec_retirement_closure::history_request::RuntimeHistoryRequest<'_, '_, '_>,
    root: &RelationalBranchRoot,
    origin: &Arc<CanonicalRequiredOutputOrigin>,
    context: &mut MapKernelContext<'_, '_>,
    lease: Option<&ExecutionResourceLease<'_>>,
) -> Result<(), Stop> {
    let envelope = runtime
        .commit(origin.commit.commit_id)?
        .ok_or("complete relation publication absent")?;
    // Prove every owned payload/birth/predecessor before incidence can classify rewiring.
    for relation in &origin.relations {
        charge(context, 1)?;
        super::super::relation_payload::validate(
            runtime,
            root,
            &envelope,
            relation.id,
            context,
            lease,
        )?;
        crate::history::data::publication_relations::validate_owned(
            root,
            relation.id,
            origin,
            &mut |version, context| runtime.version(version, context),
            context,
        )?;
    }
    crate::history::data::publication_relations::validate(
        root,
        None,
        origin,
        None,
        &mut |commit| runtime.commit(commit),
        &mut |version, context| runtime.version(version, context),
        runtime.runtime.history.catalog_len(),
        lease,
        context,
    )
}
