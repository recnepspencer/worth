use crate::durability::initial_schema_installation_denial::{
    RelationalInitialSchemaInstallationDenial, RelationalInitialSchemaInstallationDenialKind,
};
use crate::schema::data::SchemaRegistryError;
use worth_relational_durability::internal_port::durability::initial_schema_installation_denial::RelationalInitialSchemaInstallationDenialOwnerPort as _;
pub(super) fn inventory_compilation_denial(
    cause: crate::validation::CustomInvariantCompilationDenial,
) -> RelationalInitialSchemaInstallationDenial {
    use crate::validation::CustomInvariantCompilationDenial as Cause;
    use RelationalInitialSchemaInstallationDenialKind as Kind;
    match cause {
        Cause::Source(cause) => RelationalInitialSchemaInstallationDenial::new(
            Kind::ManagedIndexSourceRejected,
            format!("{cause:?}"),
        ),
        Cause::Duplicate(cause) => RelationalInitialSchemaInstallationDenial::new(
            Kind::DuplicateCustomInvariant,
            cause.identity.rule_id.as_str(),
        ),
        Cause::Execution(cause) => RelationalInitialSchemaInstallationDenial::new(
            Kind::Request(cause),
            format!("{cause:?}"),
        ),
        Cause::CapacityOverflow => RelationalInitialSchemaInstallationDenial::new(
            Kind::Request(crate::execution::RelationalExecutionDenialCause::MemoryOverflow),
            "compiled invariant inventory capacity overflow",
        ),
    }
}
pub(super) fn schema_denial(
    error: SchemaRegistryError,
) -> RelationalInitialSchemaInstallationDenial {
    RelationalInitialSchemaInstallationDenial::new(
        RelationalInitialSchemaInstallationDenialKind::SchemaRejected,
        error.detail,
    )
}

pub(in crate::runtime) fn branch_transition_denial(
    denial: crate::branch::RelationalBranchCellDenial,
) -> RelationalInitialSchemaInstallationDenial {
    let kind = match denial {
        crate::branch::RelationalBranchCellDenial::RetentionCapacityExhausted => {
            RelationalInitialSchemaInstallationDenialKind::RetentionCapacityExhausted
        }
        crate::branch::RelationalBranchCellDenial::RetentionIdentityExhausted => {
            RelationalInitialSchemaInstallationDenialKind::RetentionIdentityExhausted
        }
        crate::branch::RelationalBranchCellDenial::RetentionOwnerUnavailable => {
            RelationalInitialSchemaInstallationDenialKind::RetentionOwnerUnavailable
        }
        crate::branch::RelationalBranchCellDenial::RetentionRootSetTooLarge => {
            RelationalInitialSchemaInstallationDenialKind::RetentionRootSetTooLarge
        }
        _ => RelationalInitialSchemaInstallationDenialKind::BranchTransitionRejected,
    };
    RelationalInitialSchemaInstallationDenial::new(
        kind,
        format!("empty branch schema transition failed: {denial:?}"),
    )
}

pub(super) fn original_capture_denial(
    cause: crate::runtime::OriginalBranchSchemaInputsDenial,
) -> RelationalInitialSchemaInstallationDenial {
    use crate::runtime::OriginalBranchSchemaInputsDenial as Cause;
    let kind = match cause {
        Cause::Request(cause) => {
            RelationalInitialSchemaInstallationDenialKind::Request(cause.into())
        }
        Cause::SizeOverflow => RelationalInitialSchemaInstallationDenialKind::Request(
            crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow,
        ),
        Cause::OriginalPrefixMismatch => {
            RelationalInitialSchemaInstallationDenialKind::InitialInputsRejected
        }
    };
    RelationalInitialSchemaInstallationDenial::new(kind, "original initialization capture refused")
}
