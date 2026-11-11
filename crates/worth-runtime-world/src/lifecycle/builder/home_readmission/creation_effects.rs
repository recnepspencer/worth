//! Every actual Native branch effect joins its one original World creation.
use crate::durability::{
    decode_association, is_world_association, WorldDurableHomeBinding, WorldReadmissionDenial as D,
    WorldRecoveredJournalState,
};
use crate::durability::{
    WorldCreatedBranchOwnerEvidence as Owners, WorldRecordedRelationalCreationPlan as Plan,
};
use std::ops::ControlFlow;
use worth_execution::ExecutionRequest;
use worth_foundational::facade::{
    CanonicalDigestId, DurableOwnerFamily, UncleanRecoveryUnsupported,
};
use worth_relational::facade::{
    branch::RelationalBranchTransition as Transition,
    durability::RecoveredRelationalRuntimeAuthority, runtime::RelationalRuntime,
};

pub(super) fn validate(
    native: &RelationalRuntime,
    authority: &RecoveredRelationalRuntimeAuthority,
    image: &WorldRecoveredJournalState,
    binding: &WorldDurableHomeBinding,
    request: &ExecutionRequest<'_, '_>,
) -> Result<(), D> {
    let result = authority
        .inspect_recovered_associated_branch_transitions(request, native, |effect| {
            let bytes = effect
                .caller_association()
                .expect("associated visitor")
                .as_bytes();
            if !is_world_association(bytes) {
                return ControlFlow::Continue(());
            }
            let association = match decode_association(bytes, binding.owner().home()) {
                Ok(value) => value,
                Err(_) => return ControlFlow::Break(D::MalformedNativeAssociation),
            };
            if association.attempt().owner() != binding.owner() {
                return ControlFlow::Break(D::ForeignOwner);
            }
            let Some(digest) = effect.recorded_artifact_digest() else {
                return ControlFlow::Break(D::CreationEvidenceMismatch);
            };
            let unclean = || {
                D::UncleanRecoveryUnsupported(UncleanRecoveryUnsupported {
                    owner: DurableOwnerFamily::Relational,
                    effect: CanonicalDigestId::new(digest),
                })
            };
            let Some(original) = image.creations.get(&association.attempt()) else {
                return ControlFlow::Break(unclean());
            };
            let intent = original.intent();
            if intent.association != association {
                return ControlFlow::Break(D::ConflictingCreationIntent);
            }
            let Plan::ForkExact {
                target,
                canonical_before,
                transition_before,
            } = &intent.relational
            else {
                return ControlFlow::Break(D::CreationEvidenceMismatch);
            };
            if effect.sequence() <= *transition_before || effect.follows() < *canonical_before {
                return ControlFlow::Break(D::CreationEvidenceMismatch);
            }
            match effect.transition() {
                Transition::Fork { target: actual, .. } if actual == target => {}
                Transition::Fork { .. }
                | Transition::RetainedFork { .. }
                | Transition::Archive { .. }
                | Transition::DeletionRequested { .. }
                | Transition::Delete { .. } => {
                    return ControlFlow::Break(D::CreationEvidenceMismatch)
                }
            }
            let Some(selected) = image.created.get(&association.attempt()) else {
                return ControlFlow::Break(unclean());
            };
            let recorded = match &selected.created().owners {
                Owners::Relational(recorded)
                | Owners::RelationalAndSignal {
                    relational: recorded,
                    ..
                } => recorded,
                Owners::Reused | Owners::Signal(_) => {
                    return ControlFlow::Break(D::CreationEvidenceMismatch)
                }
            };
            if recorded.sequence != effect.sequence() || recorded.digest != digest {
                return ControlFlow::Break(D::CreationEvidenceMismatch);
            }
            ControlFlow::Continue(())
        })
        .map_err(D::NativeForkDiscovery)?;
    if let ControlFlow::Break(cause) = result {
        return Err(cause);
    }
    for (attempt, original) in &image.creations {
        if !image.created.contains_key(attempt) && !image.creation_releases.contains_key(attempt) {
            return Err(D::UncleanRecoveryUnsupported(UncleanRecoveryUnsupported {
                owner: DurableOwnerFamily::World,
                effect: CanonicalDigestId::new(*original.digest()),
            }));
        }
    }
    Ok(())
}
