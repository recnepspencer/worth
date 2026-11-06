use worth_relational::facade::runtime::RelationalInitialSchemaInstallationDenialKind;

use {crate::domain_computation::graph_layout::denial::WorthQueryPrimaryGraphInstallationDenial, crate::domain_computation::graph_layout::denial::WorthQueryPrimaryGraphInstallationDenialKind};

pub(super) fn map_initial_schema_installation_denial(
    denial: worth_relational::facade::runtime::RelationalInitialSchemaInstallationDenial,
) -> WorthQueryPrimaryGraphInstallationDenial {
    let kind = match denial.kind() {
        RelationalInitialSchemaInstallationDenialKind::RuntimeAlreadyCommitted => {
            WorthQueryPrimaryGraphInstallationDenialKind::RelationalRuntimeAlreadyPublished
        }
        RelationalInitialSchemaInstallationDenialKind::InitialInvariantsAlreadySealed
        | RelationalInitialSchemaInstallationDenialKind::DuplicateCustomInvariant
        | RelationalInitialSchemaInstallationDenialKind::RecoveredAuthorityMismatch => {
            WorthQueryPrimaryGraphInstallationDenialKind::InvariantInstallationReceiptMismatch
        }
        RelationalInitialSchemaInstallationDenialKind::SchemaRejected
        | RelationalInitialSchemaInstallationDenialKind::ManagedIndexSourceRejected
        | RelationalInitialSchemaInstallationDenialKind::BranchTransitionRejected => {
            WorthQueryPrimaryGraphInstallationDenialKind::RelationalSchemaRejected
        }
        RelationalInitialSchemaInstallationDenialKind::RetentionCapacityExhausted => {
            WorthQueryPrimaryGraphInstallationDenialKind::RetentionCapacityExhausted
        }
        RelationalInitialSchemaInstallationDenialKind::RetentionIdentityExhausted => {
            WorthQueryPrimaryGraphInstallationDenialKind::RetentionIdentityExhausted
        }
        RelationalInitialSchemaInstallationDenialKind::RetentionOwnerUnavailable => {
            WorthQueryPrimaryGraphInstallationDenialKind::RetentionOwnerUnavailable
        }
        RelationalInitialSchemaInstallationDenialKind::RetentionRootSetTooLarge => {
            WorthQueryPrimaryGraphInstallationDenialKind::RetentionRootSetTooLarge
        }
    };
    WorthQueryPrimaryGraphInstallationDenial::new(kind, denial.detail())
}
