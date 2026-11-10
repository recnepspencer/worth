//! Pin DATA joins the protected historical target before live custody is minted.
use super::*;
use worth_foundational::facade::{CorrectionMaterialPinOwner, DurableCompositeCommitIdentity};
pub(super) fn admit(
    request: &ExecutionRequest<'_, '_>,
    originals: &WorldProgramActivationHistory,
    catalog: &MaterialCatalog,
    native: &worth_relational::facade::runtime::RelationalRuntime,
    provider: &WorthQueryPrimaryGraphProvider,
) -> Result<(), Denial> {
    let prepared_keys = catalog
        .pins
        .values()
        .filter_map(|row| match row.owner {
            CorrectionMaterialPinOwner::PreparedCommand { key } => Some(key),
            CorrectionMaterialPinOwner::Publication { .. } => None,
        })
        .collect::<std::collections::BTreeSet<_>>();
    if !prepared_keys.is_empty() {
        // A decoded prepared row never proves absence of an unresolved original.
        for original in originals.original_states() {
            match original.fate() {
                worth_foundational::facade::WorldOriginalFate::Published
                | worth_foundational::facade::WorldOriginalFate::Released => {}
                worth_foundational::facade::WorldOriginalFate::Open
                | worth_foundational::facade::WorldOriginalFate::Continued => {
                    return Err(Denial::PublicationPending)
                }
            }
        }
        let layout = provider.graph.layout.provider_idempotency();
        for subject in native
            .durability_authority()
            .discover_all_associated_effects(request)
            .map_err(super::native_source::denial)?
        {
            let effect = native
                .durability_authority()
                .readmit_associated_effect(request, &subject)
                .map_err(super::native_source::denial)?;
            for creation in effect.semantic_result().created_entities() {
                if creation.key.kind != layout.entity_kind {
                    continue;
                }
                let row = effect
                    .created_entity(request, creation.assigned)
                    .map_err(super::native_source::denial)?
                    .ok_or(Denial::OriginalBindingUnavailable)?;
                let binding =
                    super::super::idempotency::snapshot_resolution::original_binding::read(
                        &row, layout,
                    )
                    .map_err(|_| Denial::OriginalBindingUnavailable)?;
                if prepared_keys.contains(&binding.key) {
                    return Err(Denial::PublicationPending);
                }
            }
        }
    }
    for row in catalog.pins.values() {
        request.consult().map_err(Denial::Execution)?;
        let target = &row.prepared.target;
        let commit = DurableCompositeCommitIdentity::from_recorded(target.owner, target.commit);
        let head = originals
            .recorded_entry(commit)
            .ok_or(Denial::TargetChanged)?;
        if head.branch().name() != target.branch
            || head.incarnation().ordinal() != target.incarnation
            || head.generation().get() != target.generation
        {
            return Err(Denial::TargetChanged);
        }
        let mut observed = originals
            .original_observations()
            .filter(|original| original.identity() == commit);
        let native = observed.next().ok_or(Denial::TargetChanged)?;
        if observed.next().is_some() || native.relational_commit() != target.relational.commit_id {
            return Err(Denial::TargetChanged);
        }
        match row.owner {
            CorrectionMaterialPinOwner::PreparedCommand { key } => {
                if key != row.prepared.key {
                    return Err(Denial::PinOwnerMismatch);
                }
            }
            CorrectionMaterialPinOwner::Publication { original } => {
                let opened = originals
                    .original_state(original)
                    .ok_or(Denial::OriginalMismatch)?;
                if opened.predecessor() != commit
                    || opened.program_revision().bytes() != &target.program
                    || opened.association().branch().name() != target.branch
                    || opened.association().incarnation().ordinal() != target.incarnation
                {
                    return Err(Denial::PinOwnerMismatch);
                }
            }
        }
    }
    Ok(())
}
