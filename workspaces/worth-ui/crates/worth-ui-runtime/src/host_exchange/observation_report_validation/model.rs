use worth_ui_host_contract::{
    UiHostObservationCanonicalCore, UiHostObservationCoalescingIdentity, UiHostObservationFamily,
    UiHostObservationIntegrity, UiHostObservationReport, UiHostObservationSequenceRange,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiHostObservationFrameRelation {
    CurrentPresented,
    SupersededPresented,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiHostObservationDisposition {
    Retained,
    Coalesced {
        replaced: UiHostObservationSequenceRange,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiHostObservationBatchDisposition {
    Complete,
    Coalesced {
        family: UiHostObservationFamily,
        replaced: UiHostObservationSequenceRange,
        survivor: UiHostObservationCoalescingIdentity,
    },
    Overflow {
        family: UiHostObservationFamily,
        affected: UiHostObservationSequenceRange,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiValidatedHostObservationReport {
    report: UiHostObservationReport,
    disposition: UiHostObservationDisposition,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiValidatedHostObservationBatch {
    core: UiHostObservationCanonicalCore,
    relation: UiHostObservationFrameRelation,
    disposition: UiHostObservationBatchDisposition,
    reports: Box<[UiValidatedHostObservationReport]>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiDuplicateHostObservationBatch {
    sequences: UiHostObservationSequenceRange,
    integrity: UiHostObservationIntegrity,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiQuarantinedHostObservationBatch {
    core: UiHostObservationCanonicalCore,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiHostObservationReportDenial {
    HostInputRetentionExhausted,
    Shutdown,
    ForeignProtocol,
    ForeignHostSession,
    MalformedBatch,
    IntegrityMismatch,
    SequenceMustBeginAtOne,
    SequenceGap,
    SequenceReordered,
    SequenceOverlap,
    SequenceExhausted,
    UnsupportedCoalescing(UiHostObservationFamily),
    CoalescingIdentityMismatch,
    LosslessOverflow(UiHostObservationFamily),
    UnknownFrame,
    ExpiredFrame,
    RejectedFrame,
    NeverPresentedFrame,
    BindingNotPresented,
    PresentationEpochMismatch,
    MountedInstanceNotPresented,
    NodeReceiptMismatch,
    LocalCapacityExceeded(UiHostObservationFamily),
    GlobalCapacityExceeded(UiHostObservationFamily),
    FrameTransitionInFlight,
    ObservationBasisCapacityExceeded {
        required_leases: usize,
        required_structural_bytes: usize,
        budget: crate::mounting::UiMountedRetentionClassBudget,
    },
    ObservationBasisAccountingOverflow,
    QuarantineCountCapacityExceeded,
    QuarantineByteCapacityExceeded,
    QuarantineAccountingOverflow,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UiHostObservationReportOutcome {
    Validated(UiValidatedHostObservationBatch),
    Duplicate(UiDuplicateHostObservationBatch),
    Quarantined(UiQuarantinedHostObservationBatch),
    Denied(UiHostObservationReportDenial),
}

impl UiValidatedHostObservationReport {
    pub(crate) fn new(
        report: UiHostObservationReport,
        disposition: UiHostObservationDisposition,
    ) -> Self {
        Self {
            report,
            disposition,
        }
    }

    pub fn report(&self) -> &UiHostObservationReport {
        &self.report
    }

    pub const fn disposition(&self) -> UiHostObservationDisposition {
        self.disposition
    }
}

impl UiValidatedHostObservationBatch {
    pub(crate) fn new(
        core: UiHostObservationCanonicalCore,
        relation: UiHostObservationFrameRelation,
        disposition: UiHostObservationBatchDisposition,
        reports: Vec<UiValidatedHostObservationReport>,
    ) -> Self {
        Self {
            core,
            relation,
            disposition,
            reports: reports.into_boxed_slice(),
        }
    }

    pub const fn canonical_core(&self) -> UiHostObservationCanonicalCore {
        self.core
    }

    pub const fn frame_relation(&self) -> UiHostObservationFrameRelation {
        self.relation
    }

    pub const fn disposition(&self) -> UiHostObservationBatchDisposition {
        self.disposition
    }

    pub fn reports(&self) -> &[UiValidatedHostObservationReport] {
        &self.reports
    }
}

impl UiDuplicateHostObservationBatch {
    pub(crate) const fn new(
        sequences: UiHostObservationSequenceRange,
        integrity: UiHostObservationIntegrity,
    ) -> Self {
        Self {
            sequences,
            integrity,
        }
    }

    pub const fn sequences(self) -> UiHostObservationSequenceRange {
        self.sequences
    }

    pub const fn integrity(self) -> UiHostObservationIntegrity {
        self.integrity
    }
}

impl UiQuarantinedHostObservationBatch {
    pub(crate) const fn new(core: UiHostObservationCanonicalCore) -> Self {
        Self { core }
    }

    pub const fn canonical_core(self) -> UiHostObservationCanonicalCore {
        self.core
    }
}
