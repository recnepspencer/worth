use worth_relational::facade::runtime::RelationalInitialSchemaInstallationDenialKind;

use super::{
    WorthQueryPrimaryGraphInstallationDenial, WorthQueryPrimaryGraphInstallationDenialKind,
};

pub(super) fn map_initial_schema_installation_denial(
    denial: worth_relational::facade::runtime::RelationalInitialSchemaInstallationDenial,
) -> WorthQueryPrimaryGraphInstallationDenial {
    let kind = match denial.kind() {
        RelationalInitialSchemaInstallationDenialKind::InitialInputsRejected => {
            WorthQueryPrimaryGraphInstallationDenialKind::NativeInitialInputsRejected
        }
        RelationalInitialSchemaInstallationDenialKind::InitialPublicationBusy => {
            WorthQueryPrimaryGraphInstallationDenialKind::NativeInitialPublicationBusy
        }
        RelationalInitialSchemaInstallationDenialKind::InitialPublicationRefused => {
            WorthQueryPrimaryGraphInstallationDenialKind::NativeInitialPublicationRefused
        }
        RelationalInitialSchemaInstallationDenialKind::InitialPublicationUnresolved => {
            WorthQueryPrimaryGraphInstallationDenialKind::NativeInitialPublicationUnresolved
        }
        RelationalInitialSchemaInstallationDenialKind::Request(cause) => {
            match super::provider::relational_execution_denial::relational_execution_kind(
                cause, None,
            ) {
                Ok(kind) => WorthQueryPrimaryGraphInstallationDenialKind::ExecutionDenied { kind },
                Err(kind) => {
                    WorthQueryPrimaryGraphInstallationDenialKind::ExecutionControlStopped { kind }
                }
            }
        }
        RelationalInitialSchemaInstallationDenialKind::RuntimeAlreadyCommitted => {
            WorthQueryPrimaryGraphInstallationDenialKind::RelationalRuntimeAlreadyPublished
        }
        RelationalInitialSchemaInstallationDenialKind::InitialInvariantsAlreadySealed
        | RelationalInitialSchemaInstallationDenialKind::DuplicateCustomInvariant
        | RelationalInitialSchemaInstallationDenialKind::RecoveredAuthorityMismatch
        | RelationalInitialSchemaInstallationDenialKind::FreshSourceMismatch => {
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
        .with_native_initialization(denial)
}
