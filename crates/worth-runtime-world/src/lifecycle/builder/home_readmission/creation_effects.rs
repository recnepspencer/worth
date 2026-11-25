//! Every actual Native branch effect joins its one original World creation.
use crate::durability::{
    decode_association, is_world_association, WorldDurableHomeBinding, WorldReadmissionDenial as D,
    WorldRecoveredJournalState,
};
use crate::durability::{
    WorldCreatedBranchOwnerEvidence as Owners, WorldRecordedRelationalCreationPlan as Plan,
    WorldRelationalForkEvidence,
};
use std::ops::ControlFlow;
use worth_execution::{ExecutionOrderedMap, ExecutionRequest};
use worth_foundational::facade::DurableCompositePublicationAttemptIdentity as Attempt;
use worth_relational::facade::{
    branch::RelationalBranchTransition as Transition,
    durability::RecoveredRelationalRuntimeAuthority, runtime::RelationalRuntime,
};

/// Native's own recovered inventory of forks whose original creation has
/// neither a selection nor a release. Absence here is Native's no-effect proof.
#[derive(Default)]
pub(super) struct UnresolvedCreationForks {
    forks: ExecutionOrderedMap<Attempt, WorldRelationalForkEvidence>,
}
impl UnresolvedCreationForks {
    pub(super) fn get(&self, attempt: &Attempt) -> Option<&WorldRelationalForkEvidence> {
        self.forks.get(attempt)
    }
}

pub(super) fn validate(
    native: &RelationalRuntime,
    authority: &RecoveredRelationalRuntimeAuthority,
    image: &WorldRecoveredJournalState,
    binding: &WorldDurableHomeBinding,
    request: &ExecutionRequest<'_, '_>,
) -> Result<UnresolvedCreationForks, D> {
    let mut unresolved = UnresolvedCreationForks::default();
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
            // A fork with no recorded creation intent joins no original.
            let Some(original) = image.creations.get(&association.attempt()) else {
                return ControlFlow::Break(D::MissingCreationIntent);
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
            // A durable release is published only for never-submitted owners.
            if image.creation_releases.contains_key(&association.attempt()) {
                return ControlFlow::Break(D::CreationEvidenceMismatch);
            }
            let evidence = WorldRelationalForkEvidence {
                sequence: effect.sequence(),
                digest,
            };
            let Some(selected) = image.created.get(&association.attempt()) else {
                return admit_unresolved(
                    &mut unresolved,
                    &association.attempt(),
                    evidence,
                    request,
                );
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
            if *recorded != evidence {
                return ControlFlow::Break(D::CreationEvidenceMismatch);
            }
            ControlFlow::Continue(())
        })
        .map_err(D::NativeForkDiscovery)?;
    match result {
        ControlFlow::Continue(()) => Ok(unresolved),
        ControlFlow::Break(cause) => Err(cause),
    }
}

/// One original creation owns at most one actual Native fork.
fn admit_unresolved(
    unresolved: &mut UnresolvedCreationForks,
    attempt: &Attempt,
    evidence: WorldRelationalForkEvidence,
    request: &ExecutionRequest<'_, '_>,
) -> ControlFlow<D> {
    if unresolved.forks.contains_key(attempt) {
        return ControlFlow::Break(D::DuplicateNativeEffect);
    }
    match unresolved
        .forks
        .insert_cloned_native_for_request(attempt, &evidence, *request)
    {
        Ok(_) => ControlFlow::Continue(()),
        Err(cause) => ControlFlow::Break(D::Medium(
            crate::durability::WorldDurableMediumDenial::Ordered(cause),
        )),
    }
}
