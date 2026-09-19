use worth_relational::facade::runtime::RelationalInitialSchemaInstallationDenialKind;

use super::{
    WorthQueryPrimaryGraphInstallationDenial, WorthQueryPrimaryGraphInstallationDenialKind,
};

pub(super) fn map_initial_schema_installation_denial(
    denial: worth_relational::facade::runtime::RelationalInitialSchemaInstallationDenial,
) -> WorthQueryPrimaryGraphInstallationDenial {
    let kind = match denial.kind() {
        RelationalInitialSchemaInstallationDenialKind::RuntimeAlreadyCommitted => {
            WorthQueryPrimaryGraphInstallationDenialKind::RelationalRuntimeAlreadyPublished
        }
        RelationalInitialSchemaInstallationDenialKind::InitialInvariantsAlreadySealed
        | RelationalInitialSchemaInstallationDenialKind::DuplicateCustomInvariant => {
            WorthQueryPrimaryGraphInstallationDenialKind::InvariantInstallationReceiptMismatch
        }
        RelationalInitialSchemaInstallationDenialKind::SchemaRejected
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
