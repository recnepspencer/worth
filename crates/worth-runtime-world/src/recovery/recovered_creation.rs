//! One cold-recovered branch creation: the SAME original intent, identities
//! and target name, with the fate its owners' recovered inventories decided.
use crate::branch::registry::ProductBranchRegistryReservation;
use crate::branch::ProductBranchObservation;
use crate::durability::{
    WorldCommittedPublication, WorldCreationIntentRecord, WorldDurableArtifact,
    WorldDurableMediumBinding,
};
use crate::identity::ProductUnpublishedOwnerEffectsIdentity;
use std::sync::{Arc, Mutex};
use worth_relational::facade::durability::ReadmittedRelationalAssociatedFork;
use worth_signal::facade::durability::{SignalReadmittedSourcePoint, SignalRecordedSourcePoint};

/// The only decision the recorded owner facts admit for an unresolved creation.
#[derive(Debug)]
pub(crate) enum RecoveredCreationFate {
    /// Neither owner's recovered inventory holds an effect: release only.
    Release,
    /// Every required owner fork is durable: the original continues only.
    Continue(RecoveredCreationForks),
    /// Native's fork is durable and Signal's planned fork is not. The
    /// performed effect may only continue: recovery performs the original's
    /// Signal fork once, at the recorded source point, then continues.
    CompleteSignalFork(RecoveredSignalForkCompletion),
}

/// The original's outstanding Signal fork beside its admitted Native fork.
#[derive(Debug)]
pub(crate) struct RecoveredSignalForkCompletion {
    pub(crate) native: ReadmittedRelationalAssociatedFork,
    pub(crate) phase: RecoveredSignalForkPhase,
}

#[derive(Debug)]
pub(crate) enum RecoveredSignalForkPhase {
    /// No Signal fork has begun in this process.
    Unbegun,
    /// The one Signal fork task, driven again until Signal reports its fate.
    Driving(Arc<crate::publication::durable_signal::WorldDurableSignalAdvance>),
    /// Signal could not report the fork's fate here. It is never begun again
    /// in this process; the next reopen reads Signal's own inventory.
    ReopenRequired,
}

/// Owner-admitted forks of the original; each exists exactly when its plan forks.
#[derive(Debug)]
pub(crate) struct RecoveredCreationForks {
    pub(crate) native: Option<ReadmittedRelationalAssociatedFork>,
    pub(crate) signal: Option<Arc<SignalReadmittedSourcePoint>>,
}

/// Exclusive custody of the unresolved original. A resolver moves it out of
/// the record; any other request finds it absent and refuses as busy.
#[derive(Debug)]
pub(crate) struct RecoveredCreationCustody {
    pub(crate) fate: RecoveredCreationFate,
    /// Fences the original target name until the original is decided.
    pub(crate) destination: ProductBranchRegistryReservation,
    /// The decision artifact, built once so a retry publishes the same bytes.
    pub(crate) decision: Option<WorldDurableArtifact>,
    /// The durable decision, once acknowledged; it is never published twice.
    pub(crate) committed: Option<WorldCommittedPublication>,
}

#[derive(Debug)]
pub(crate) struct RecoveredWorldCreationRecord {
    pub(crate) identity: ProductUnpublishedOwnerEffectsIdentity,
    pub(crate) intent: WorldCreationIntentRecord,
    pub(crate) source: ProductBranchObservation,
    pub(crate) source_signal: SignalRecordedSourcePoint,
    pub(crate) binding: Arc<WorldDurableMediumBinding>,
    pub(crate) custody: Mutex<Option<RecoveredCreationCustody>>,
    pub(crate) metadata_bytes: usize,
}

pub(crate) struct RecoveredWorldCreationInputs {
    pub(crate) identity: ProductUnpublishedOwnerEffectsIdentity,
    pub(crate) intent: WorldCreationIntentRecord,
    pub(crate) source: ProductBranchObservation,
    pub(crate) source_signal: SignalRecordedSourcePoint,
    pub(crate) binding: Arc<WorldDurableMediumBinding>,
    pub(crate) fate: RecoveredCreationFate,
    pub(crate) destination: ProductBranchRegistryReservation,
}

impl RecoveredWorldCreationRecord {
    pub(crate) fn new(input: RecoveredWorldCreationInputs, metadata_bytes: usize) -> Arc<Self> {
        Arc::new(Self {
            identity: input.identity,
            intent: input.intent,
            source: input.source,
            source_signal: input.source_signal,
            binding: input.binding,
            custody: Mutex::new(Some(RecoveredCreationCustody {
                fate: input.fate,
                destination: input.destination,
                decision: None,
                committed: None,
            })),
            metadata_bytes,
        })
    }

    /// Move the exclusive custody to one resolver; `None` while another holds it.
    pub(crate) fn take_custody(&self) -> Option<RecoveredCreationCustody> {
        self.custody
            .lock()
            .unwrap_or_else(|cause| cause.into_inner())
            .take()
    }

    /// Return custody after a resolution stopped before its terminal step.
    pub(crate) fn restore_custody(&self, custody: RecoveredCreationCustody) {
        let mut slot = self
            .custody
            .lock()
            .unwrap_or_else(|cause| cause.into_inner());
        assert!(slot.is_none(), "one resolver holds the original");
        *slot = Some(custody);
    }
}
