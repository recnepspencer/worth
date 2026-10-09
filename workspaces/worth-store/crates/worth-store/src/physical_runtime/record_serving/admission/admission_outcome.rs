use crate::physical_runtime::{
    AbortedRuntime, MediaOwnedPhysicalRuntime, MediaShutdownOutcome, RuntimeIdentity,
};

use super::super::{
    RecordBootstrapDenial, RecordBootstrapFailure, RecordServingRebindReason,
    RecordServingStaleReason, ServingPhysicalRuntime,
};

pub type RecordServingAdmissionOutcome<Denial> = worth_proof::ProofOutcome<
    ServingPhysicalRuntime,
    Denial,
    RecordServingAdmissionDeferred,
    RecordServingAdmissionStale,
    RecordServingAdmissionRebindRequired,
    RecordServingAdmissionInspectionRequired,
>;

pub type RecordStoreInitializationOutcome =
    RecordServingAdmissionOutcome<RecordStoreInitializationDenial>;
pub type RecordStoreOpenOutcome = RecordServingAdmissionOutcome<RecordStoreOpenDenial>;

pub struct RecordStoreInitializationDenial {
    runtime: MediaOwnedPhysicalRuntime,
    reason: RecordBootstrapDenial,
}

impl RecordStoreInitializationDenial {
    pub(in crate::physical_runtime::record_serving) const fn new(
        runtime: MediaOwnedPhysicalRuntime,
        reason: RecordBootstrapDenial,
    ) -> Self {
        Self { runtime, reason }
    }

    pub fn reason(&self) -> RecordBootstrapDenial {
        self.reason.clone()
    }

    pub fn into_runtime(self) -> MediaOwnedPhysicalRuntime {
        self.runtime
    }
}

pub struct RecordStoreOpenDenial {
    runtime: MediaOwnedPhysicalRuntime,
    reason: RecordBootstrapDenial,
}

impl RecordStoreOpenDenial {
    pub(in crate::physical_runtime::record_serving) const fn new(
        runtime: MediaOwnedPhysicalRuntime,
        reason: RecordBootstrapDenial,
    ) -> Self {
        Self { runtime, reason }
    }

    pub fn reason(&self) -> RecordBootstrapDenial {
        self.reason.clone()
    }

    pub fn into_runtime(self) -> MediaOwnedPhysicalRuntime {
        self.runtime
    }
}

pub enum RecordServingAdmissionDeferred {}
pub struct RecordServingAdmissionStale {
    runtime: MediaOwnedPhysicalRuntime,
    reason: RecordServingStaleReason,
}

impl RecordServingAdmissionStale {
    pub(in crate::physical_runtime::record_serving) const fn new(
        runtime: MediaOwnedPhysicalRuntime,
        reason: RecordServingStaleReason,
    ) -> Self {
        Self { runtime, reason }
    }
    pub const fn reason(&self) -> RecordServingStaleReason {
        self.reason
    }
    pub fn into_runtime(self) -> MediaOwnedPhysicalRuntime {
        self.runtime
    }
}

pub struct RecordServingAdmissionRebindRequired {
    runtime: MediaOwnedPhysicalRuntime,
    reason: RecordServingRebindReason,
}

impl RecordServingAdmissionRebindRequired {
    pub(in crate::physical_runtime::record_serving) const fn new(
        runtime: MediaOwnedPhysicalRuntime,
        reason: RecordServingRebindReason,
    ) -> Self {
        Self { runtime, reason }
    }
    pub const fn reason(&self) -> RecordServingRebindReason {
        self.reason
    }
    pub fn into_runtime(self) -> MediaOwnedPhysicalRuntime {
        self.runtime
    }
}

pub struct RecordServingAdmissionInspectionRequired {
    runtime_identity: RuntimeIdentity,
    terminal: MediaShutdownOutcome<AbortedRuntime>,
    cause: RecordBootstrapFailure,
}

impl RecordServingAdmissionInspectionRequired {
    pub(in crate::physical_runtime::record_serving) const fn new(
        runtime_identity: RuntimeIdentity,
        terminal: MediaShutdownOutcome<AbortedRuntime>,
        cause: RecordBootstrapFailure,
    ) -> Self {
        Self {
            runtime_identity,
            terminal,
            cause,
        }
    }

    pub const fn runtime_identity(&self) -> RuntimeIdentity {
        self.runtime_identity
    }

    pub const fn terminal(&self) -> &MediaShutdownOutcome<AbortedRuntime> {
        &self.terminal
    }

    pub fn cause(&self) -> RecordBootstrapFailure {
        self.cause.clone()
    }
}
