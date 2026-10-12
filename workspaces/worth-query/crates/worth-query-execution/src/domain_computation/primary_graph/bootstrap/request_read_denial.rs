//! Bootstrap keeps a native read's exact execution cause at installation.
use super::super::{
    WorthQueryPrimaryGraphInstallationDenial as Denial,
    WorthQueryPrimaryGraphInstallationDenialKind as Kind,
};
pub(super) fn read(
    cause: worth_relational::facade::transactions::RelationalExecutionDenialCause,
) -> Denial {
    let kind = match super::super::provider::relational_execution_denial::relational_execution_kind(
        cause, None,
    ) {
        Ok(kind) => Kind::ExecutionDenied { kind },
        Err(kind) => Kind::ExecutionControlStopped { kind },
    };
    Denial::new(kind, "bootstrap read stopped")
}
