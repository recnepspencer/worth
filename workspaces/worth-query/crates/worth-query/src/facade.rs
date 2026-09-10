//! Public API boundary for `worth-query`.
//! External crates should import through this module rather than reaching into
//! internal crate structure directly.

mod exports_aggregate;
mod exports_application;
mod exports_certification;
mod exports_comparison;
mod exports_domain;
mod exports_domain_artifacts;
mod exports_domain_capabilities;
mod exports_domain_evidence;
mod exports_foundation;
mod exports_history;
mod exports_inspection;
mod exports_installed;
mod exports_live_capability;
mod exports_mutation;
mod exports_policy;
mod exports_preview;
mod exports_read;
mod exports_runtime;
mod exports_runtime_core;
mod exports_runtime_phase_nine;
mod exports_runtime_products;
mod exports_workflow;
mod installed_transitions;

pub mod identity_authority {
    pub use crate::identity_authority::*;
}

pub mod consumer_kit {
    pub use crate::consumer_kit::{
        advance_test_workspace_domain_installation_generation, compare_test_backend_write_receipts,
        in_memory_test_product_world_resources, in_memory_test_runtime,
        load_support_pin_contract_terminal_json_document,
        load_support_snapshot_terminal_json_document, project_support_snapshot,
        project_workspace_support_snapshot, support_pinning_contract, EvidenceReport,
        EvidenceReportDeclaration, EvidenceReportError, EvidenceReportErrorKind,
        EvidenceReportField, EvidenceReportFieldKind, EvidenceReportFieldParticipation,
        EvidenceReportFieldValue, EvidenceReportScope, WorthQueryControlledTestWorkspace,
        WorthQueryExternalSupportPinContractTerminalJsonDocument,
        WorthQueryExternalSupportSnapshotTerminalJsonDocument,
        WorthQueryInMemoryTestRuntimeBuilder, WorthQueryObservedSupportPin,
        WorthQueryPinnedSupportStatus, WorthQueryPinnedTeachingPosture,
        WorthQuerySupportPinContract, WorthQuerySupportPinContractBuilder,
        WorthQuerySupportPinContractSchemaVersion,
        WorthQuerySupportPinContractTerminalJsonDocument, WorthQuerySupportPinDeclaration,
        WorthQuerySupportPinFinding, WorthQuerySupportPinFindingKind, WorthQuerySupportPinReport,
        WorthQuerySupportPinRequirement, WorthQuerySupportPinRequirementDraft,
        WorthQuerySupportPinningError, WorthQuerySupportPinningErrorKind,
        WorthQuerySupportSnapshot, WorthQuerySupportSnapshotError,
        WorthQuerySupportSnapshotErrorKind, WorthQuerySupportSnapshotRow,
        WorthQuerySupportSnapshotSchemaVersion, WorthQuerySupportSnapshotTerminalJsonDocument,
        WorthQueryTestBackendEquivalenceReport, WorthQueryTestBackendEquivalenceRow,
        WorthQueryTestBackendError, WorthQueryTestBackendErrorKind, WorthQueryTestBackendSchema,
        WorthQueryTestSeedReceipt, WorthQueryTestSeedRow,
    };
    pub use crate::runtime::WorthQueryRuntimeFacadeFamily;
}

/// Certification, migration, manifest, audit, and hostile-test tooling.
///
/// This namespace is intentionally separate from the ordinary product facade.
/// Production consumers should depend on `foundation`, `policy`, or `runtime`.
pub mod certification {
    pub use super::exports_certification::*;
}

pub mod foundation {
    pub use super::exports_application::*;
    pub use super::exports_foundation::*;
}

pub mod policy {
    pub use super::exports_policy::*;
}

/// Declarative one-shot read capability.
///
/// Consumers author bounded read meaning and hand the resulting declaration to
/// Query. Query owns admission, planning, execution routing, and receipt
/// construction.
pub mod read {
    pub use super::exports_read::*;
}

/// Declarative bounded aggregate capability.
pub mod aggregate {
    pub use super::exports_aggregate::*;
}

/// Declarative framework-owned live capability.
pub mod live {
    pub use super::exports_live_capability::*;
}

/// Declarative historical read capability.
pub mod history {
    pub use super::exports_history::*;
}

/// Declarative diff, lineage, and correspondence capability.
pub mod comparison {
    pub use super::exports_comparison::*;
}

/// Declarative authoritative mutation capability.
pub mod mutation {
    pub use super::exports_mutation::*;
}

/// Declarative scoped preview capability.
pub mod preview {
    pub use super::exports_preview::*;
}

/// Declarative preview, promotion, and writeback workflow capability.
pub mod workflow {
    pub use super::exports_workflow::*;
}

/// Runtime-installed domain package, handle, contribution, and operation grammar.
///
/// Downstream crates may add typed extension traits over installed handles, but
/// Query retains package identity, installation, admission, execution, receipt,
/// and diagnostic authority.
pub mod domain {
    pub use super::exports_domain::*;
    pub use super::exports_domain_artifacts::*;
    pub use super::exports_domain_capabilities::*;
    pub use super::exports_domain_evidence::*;
}

/// Common outcome navigation and declarative inspection capability.
pub mod inspection {
    pub use super::exports_inspection::*;
}

/// Ordinary runtime-installed operation progression.
///
/// This is the discoverable consumer path after a host has installed domain
/// packages and volatile providers. It exposes Query-owned transitions and
/// descriptive inspection, but no package, provider, replay, or proof
/// construction ingredients.
pub mod installed {
    pub use super::exports_installed::*;
}

pub mod runtime {
    pub use super::exports_runtime::*;
    pub use super::exports_runtime_core::*;
    pub use super::exports_runtime_phase_nine::*;
    pub use super::exports_runtime_products::*;
}

/// Ordinary product-world selection, creation, and operation progression.
pub mod product {
    pub use crate::runtime::product_branch::*;
}

#[cfg(test)]
pub(crate) use crate::query_context::{
    admit_and_scope_legacy_query_basis_context_for_test, bind_legacy_query_basis_context,
    QueryBasisContextRequest,
};
