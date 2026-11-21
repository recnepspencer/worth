//! Explicit internal contract for durable Query obligations and correction
//! journal framing.

pub mod aftermath {
    pub mod correction_material {
        pub use crate::aftermath::correction_material::*;
    }
    pub mod derivation_failure {
        pub use crate::aftermath::derivation_failure::*;
    }
    pub mod recovery_posture {
        pub use crate::aftermath::recovery_posture::*;
    }
}

pub mod query_obligations {
    pub use crate::query_obligations::{
        QueryAcceptedOutputRecord, QueryAdoptionPerformedDescription, QueryAdoptionRecord,
        QueryBranchAdoptionObligation, QueryColdObligationFold, QueryColdRelease,
        QueryColdReleaseSubject, QueryColdReleaseTerminal, QueryConditionalDecision,
        QueryConditionalDecisionRecord, QueryConditionalPosture, QueryEndedIncarnation,
        QueryManagedCleanupObligation, QueryManagedCleanupOriginal, QueryManagedCleanupRecord,
        QueryManagedCleanupRelease, QueryManagedExecutionRelease, QueryManagedRunObligation,
        QueryManagedRunPosture, QueryManagedRunRecord, QueryManagedRunRecovery,
        QueryManagedRunResources, QueryManagedWorkflowCleanupRoute, QueryManagedYieldRecoveryState,
        QueryObligationDenial, QueryObligationFrame, QueryObligationLedger,
        QueryObligationPublicationGenerations, QueryObligationPublicationOutcome,
        QueryObligationRecord, QueryObligationsMedium, QueryOriginalOperationData,
        QueryOriginalOperationObligation, QueryOriginalOperationPosition,
        QueryOriginalOperationRecord, QueryOutputDemandObligation, QueryOutputDemandRecord,
        QueryOutputSourceObligation, QueryOutputSourceRecord, QueryOwnedAsyncContinuation,
        QueryOwnedAsyncDefinition, QueryOwnedAsyncObligation, QueryOwnedAsyncPosture,
        QueryOwnedAsyncRecord, QueryOwnedAsyncRequestGeneration, QueryOwnedAsyncRetention,
        QueryOwnedAsyncRetryCause, QueryOwnedAsyncTerminal, QueryProgramSupportObligation,
        QueryProgramSupportRecord, QueryResourceHandoff, QueryResourceObligation,
        QueryResourceOriginal, QueryResourceRecord, QueryResourceRelease, QueryResourceResultKind,
        QueryStoppedBranchSetObligation, QUERY_OBLIGATION_FRAME_MAX_BYTES,
    };
}

/// Internal read-only ledger fold inspection for Query's obligation history
/// owner. Never re-exported on public execution facades.
pub mod ledger_inspect {
    pub use crate::query_obligations::ledger_inspect::*;
}
