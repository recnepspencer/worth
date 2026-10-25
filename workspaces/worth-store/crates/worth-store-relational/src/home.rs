//! A Relational home: one Relational runtime whose only durable medium is one
//! Worth Store.
//!
//! The home owns the durable lifecycle. It creates a Store only where none
//! exists, otherwise recovers it through Store's production recovery entry,
//! reads the home's descriptor, checkpoint and journal, and hands them to
//! Relational for readmission before the runtime serves anything. Every commit
//! the runtime makes afterwards is published to the journal before it becomes
//! live.

#[cfg(feature = "certification-test-authority")]
mod certification;
mod checkpoint;
mod close;
mod open_refusal;
mod request;
mod resolution;
mod transition_resolution;

pub use checkpoint::{RelationalStoreCheckpoint, RelationalStoreCheckpointFailure};
pub use close::RelationalStoreHomeClosed;
pub use open_refusal::RelationalStoreHomeOpenRefusal;
pub use request::RelationalStoreHomeRequest;
pub use resolution::RelationalStoreCommitFate;
pub use transition_resolution::RelationalStoreTransitionFate;

use std::path::Path;
use std::sync::Arc;

use worth_relational::facade::branch::RelationalBranchTransitionSequence;
use worth_relational::facade::durability::{
    RecoveryVerificationMode, RelationalDurableRecoverySource,
};
use worth_relational::facade::durability::{
    RelationalBranchTransitionArtifactDigest, RelationalCanonicalCommitArtifactDigest,
};
use worth_relational::facade::publication::PatchStreamPosition;
use worth_relational::facade::runtime::RelationalRuntime;
use worth_store::physical_runtime::ServingPhysicalRuntime;

use crate::home_descriptor::HomeDescriptor;
use crate::journal_medium::StoreJournalMedium;
use crate::physical_store::PhysicalOpening;
use crate::recovered_history::RelationalStoreJournalRefusal;
use crate::recovered_transitions::RelationalStoreTransitionJournalRefusal;
use crate::{
    checkpoint_record, home_descriptor, physical_store, recovered_history, recovered_transitions,
};

type Refusal = RelationalStoreHomeOpenRefusal;

pub struct RelationalStoreHome {
    /// Declared before `serving` so the runtime, which holds the journal
    /// medium, is dropped first.
    runtime: RelationalRuntime,
    serving: Arc<ServingPhysicalRuntime>,
}

impl RelationalStoreHome {
    /// Open the home at the request's location: create it if nothing is
    /// there, otherwise recover it. Service begins only after Relational has
    /// readmitted everything the Store recovered.
    pub fn open(
        request: RelationalStoreHomeRequest,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Self, Refusal> {
        Self::open_through(request, PhysicalOpening::production(), execution)
    }

    fn open_through(
        request: RelationalStoreHomeRequest,
        opening: PhysicalOpening,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Self, Refusal> {
        let (location, builder) = request.into_parts();
        let serving = open_physical_store(&location, opening)?;
        Self::admit(serving, builder, execution)
    }

    /// Attach a runtime built from `builder` to an open Store, after
    /// readmitting the home's descriptor, checkpoint and journal.
    fn admit(
        serving: ServingPhysicalRuntime,
        builder: worth_relational::facade::runtime::RelationalRuntimeBuilder,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Self, Refusal> {
        let serving = Arc::new(serving);
        let medium = StoreJournalMedium::new(Arc::clone(&serving), physical_store::placement());
        let mut runtime = builder.build_durable(Box::new(medium));
        let readmitted = admit_descriptor(&serving, &runtime)
            .and_then(|()| readmit_history(&serving, &mut runtime, execution));
        match readmitted {
            Ok(()) => Ok(Self { runtime, serving }),
            Err(refusal) => {
                close::abandon(runtime, serving);
                Err(refusal)
            }
        }
    }

    /// The live runtime. Its commits are durable before they are visible.
    pub fn runtime(&self) -> &RelationalRuntime {
        &self.runtime
    }

    /// The durable fate of the commit whose artifact digest is `digest` at
    /// `position`, from the journal alone. Pass the digest a
    /// `DurabilityUnresolved` outcome carried to learn whether that commit
    /// became durable.
    pub fn resolve(
        &self,
        position: PatchStreamPosition,
        digest: RelationalCanonicalCommitArtifactDigest,
    ) -> Result<RelationalStoreCommitFate, RelationalStoreJournalRefusal> {
        resolution::resolve(&self.serving, position, digest)
    }

    /// The durable fate of the branch transition whose artifact digest is
    /// `digest` at `sequence`, from the transition journal alone. Pass the
    /// digest a `DurabilityUnresolved` branch transition carried.
    pub fn resolve_branch_transition(
        &self,
        sequence: RelationalBranchTransitionSequence,
        digest: RelationalBranchTransitionArtifactDigest,
    ) -> Result<RelationalStoreTransitionFate, RelationalStoreTransitionJournalRefusal> {
        transition_resolution::resolve(&self.serving, sequence, digest)
    }
}

fn open_physical_store(
    location: &Path,
    opening: PhysicalOpening,
) -> Result<ServingPhysicalRuntime, Refusal> {
    if request::location_is_unused(location)? {
        physical_store::initialize(location, opening).map_err(Refusal::Physical)
    } else {
        physical_store::recover(location, opening).map_err(Refusal::Physical)
    }
}

/// Require the home's descriptor to name this runtime. A Store with no
/// descriptor and no history is a home whose creation stopped before its
/// descriptor; it is completed. One with history is refused.
fn admit_descriptor(
    serving: &ServingPhysicalRuntime,
    runtime: &RelationalRuntime,
) -> Result<(), Refusal> {
    let expected = HomeDescriptor::of(runtime);
    match home_descriptor::read(serving).map_err(Refusal::Descriptor)? {
        Some(found) if found == expected => Ok(()),
        Some(found) => Err(Refusal::ForeignHome {
            expected: expected.describe(),
            found: found.describe(),
        }),
        None => {
            let has_history = checkpoint_record::read(serving)
                .map_err(Refusal::Checkpoint)?
                .is_some()
                || !recovered_history::journal_tail(serving, 0)
                    .map_err(Refusal::Journal)?
                    .is_empty()
                || !recovered_transitions::transition_tail(serving, 0)
                    .map_err(Refusal::TransitionJournal)?
                    .is_empty();
            if has_history {
                return Err(Refusal::DescriptorMissing);
            }
            home_descriptor::publish(serving, &expected).map_err(Refusal::Descriptor)
        }
    }
}

/// Hand the selected checkpoint, and the commits and branch transitions after
/// it, to Relational.
fn readmit_history(
    serving: &Arc<ServingPhysicalRuntime>,
    runtime: &mut RelationalRuntime,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<(), Refusal> {
    let checkpoint = checkpoint_record::read(serving).map_err(Refusal::Checkpoint)?;
    let covered = checkpoint
        .as_ref()
        .map_or(0, |checkpoint| checkpoint.covered().0);
    let covered_transitions = checkpoint
        .as_ref()
        .map_or(0, |checkpoint| checkpoint.covered_transitions().0);
    let tail = recovered_history::journal_tail(serving, covered).map_err(Refusal::Journal)?;
    let transitions = recovered_transitions::transition_tail(serving, covered_transitions)
        .map_err(Refusal::TransitionJournal)?;
    let source = RelationalDurableRecoverySource::new(
        checkpoint,
        tail,
        transitions,
        Arc::new(crate::cold_source::StoreColdSource::new(Arc::clone(
            serving,
        ))),
    );
    let plan = runtime
        .durability()
        .recovery_plan_from_source(
            execution,
            source,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .map_err(Refusal::Readmission)?;
    runtime
        .durability_recovery()
        .recover(execution, plan)
        .map_err(Refusal::Readmission)?;
    Ok(())
}
