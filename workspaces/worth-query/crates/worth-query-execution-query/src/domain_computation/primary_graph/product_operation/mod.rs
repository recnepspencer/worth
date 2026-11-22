pub mod application_basis;
pub mod conditional_definition;
pub mod context;
pub mod correction_source;
pub mod current_read;
pub mod document_history;
pub(crate) mod entry_internal_port;
pub mod observation_room;
pub mod operation;
pub mod query_controls;
pub mod security_basis;
pub mod shared_query_basis;

pub mod query {
    pub use super::query_controls::WorthQueryProductQueryControls;
    use super::WorthQuerySelectedProductOperation;
    pub mod retained;

    /// Preserve the original request stop instead of relabeling it as a basis mismatch.
    pub fn selection_denial(
        stop: crate::basis::WorthQueryProductBranchAdmissionDenial,
        query_name: &str,
    ) -> crate::domain_computation::primary_graph::WorthQueryApplicationQueryAdmissionDenial {
        use crate::basis::WorthQueryProductBranchAdmissionDenial as Branch;
        use crate::domain_computation::primary_graph::WorthQueryApplicationQueryAdmissionDenialKind as Kind;
        use crate::domain_computation::WorthQueryProviderSessionControlStopKind as Stop;
        let kind = match stop {
            Branch::Stopped(Err(Stop::Cancelled)) => Kind::Cancelled,
            Branch::Stopped(Err(Stop::TimedOut)) => Kind::DeadlineExceeded,
            _ => Kind::ForeignBasis,
        };
        crate::domain_computation::primary_graph::WorthQueryApplicationQueryAdmissionDenial::new(
            kind, query_name,
        )
    }
}

pub use conditional_definition::{
    WorthQueryAdmittedApplicationConditionalDefinition,
    WorthQueryApplicationConditionalDefinitionAdmissionDenial,
};
pub use context::{
    SelectedQueryOrigin, WorthQueryProductEntry, WorthQuerySelectedProductOperation,
};
pub use correction_source::WorthQueryCorrectionIntent;
pub use document_history::{
    WorthQueryDocumentHistoryDenial, WorthQueryExpectedProductBranchAdoptionDenial,
    WorthQueryHistoryContextRefreshDenial, WorthQueryHistoryNavigationDenial,
    WorthQueryHistoryNavigationEligibilityDenial, WorthQueryHistoryNodeId,
    WorthQueryHistoryOutputControlDenial, WorthQueryHistoryOutputReadDenial,
    WorthQueryHistoryPageDenial, WorthQueryHistoryStep,
};
pub use query_controls::WorthQueryProductQueryControls;
pub use security_basis::{
    SelectedPermissionSecurityStop, WorthQueryProductSecurityBasis,
    WorthQuerySelectedPermissionSecurityBasis,
};
pub use shared_query_basis::{SelectedQueryBasisRetentionStop, SharedSelectedProductOperation};
