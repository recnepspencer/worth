//! Typed outcome of the original schema publication and its readmission.
use worth_relational_contracts::internal_port as contracts;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RelationalInitialSchemaInstallationDenialKind {
    RuntimeAlreadyCommitted,
    InitialInvariantsAlreadySealed,
    DuplicateCustomInvariant,
    ManagedIndexSourceRejected,
    SchemaRejected,
    BranchTransitionRejected,
    RetentionCapacityExhausted,
    RetentionIdentityExhausted,
    RetentionOwnerUnavailable,
    RetentionRootSetTooLarge,
    RecoveredAuthorityMismatch,
    FreshSourceMismatch,
    InitialInputsRejected,
    InitialPublicationBusy,
    InitialPublicationRefused,
    InitialPublicationUnresolved,
    Request(contracts::execution::denial_cause::RelationalExecutionDenialCause),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelationalInitialSchemaInstallationDenial {
    kind: RelationalInitialSchemaInstallationDenialKind,
    detail: String,
    publication:
        Option<crate::durability::published_medium::RelationalInitialInputsPublicationStop>,
}

impl RelationalInitialSchemaInstallationDenial {
    pub(crate) fn new(
        kind: RelationalInitialSchemaInstallationDenialKind,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            detail: detail.into(),
            publication: None,
        }
    }

    pub const fn kind(&self) -> RelationalInitialSchemaInstallationDenialKind {
        self.kind
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }
}

impl std::fmt::Display for RelationalInitialSchemaInstallationDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "initial Relational schema installation denied: {:?} ({})",
            self.kind, self.detail
        )
    }
}

impl std::error::Error for RelationalInitialSchemaInstallationDenial {}

impl RelationalInitialSchemaInstallationDenial {
    pub fn publication_stop(
        &self,
    ) -> Option<&crate::durability::published_medium::RelationalInitialInputsPublicationStop> {
        self.publication.as_ref()
    }
    pub(crate) fn publication(
        cause: crate::durability::published_medium::RelationalInitialInputsPublicationStop,
    ) -> Self {
        use crate::durability::published_medium::RelationalInitialInputsPublicationStop as Stop;
        let kind = match &cause {
            Stop::Request(cause) => RelationalInitialSchemaInstallationDenialKind::Request(*cause),
            Stop::Refused(_) => {
                RelationalInitialSchemaInstallationDenialKind::InitialPublicationRefused
            }
            Stop::Unresolved(_) => {
                RelationalInitialSchemaInstallationDenialKind::InitialPublicationUnresolved
            }
        };
        Self {
            kind,
            detail: format!("{cause:?}"),
            publication: Some(cause),
        }
    }
    /// Preserve the owner's initialization refusal before any installation step.
    pub fn from_initial_inputs(
        cause: crate::durability::source_initial_inputs::RelationalInitialInputsRefusal,
    ) -> Self {
        use crate::durability::source_initial_inputs::RelationalInitialInputsRefusal as Cause;
        let kind = match &cause {
            Cause::Request(cause)
            | Cause::Source(
                worth_relational_storage::internal_port::durability::medium::read_protection::refusal::RelationalDurableOwnedReadRefusal::Request(cause),
            )
            | Cause::Native(contracts::durability::data::recovery_stop::RelationalRecoveryStop::Request(cause)) => {
                RelationalInitialSchemaInstallationDenialKind::Request(*cause)
            }
            Cause::Native(contracts::durability::data::recovery_stop::RelationalRecoveryStop::Content(cause)) => {
                RelationalInitialSchemaInstallationDenialKind::Request(
                    (*cause).into_execution_cause(),
                )
            }
            Cause::Native(contracts::durability::data::recovery_stop::RelationalRecoveryStop::Allocation(_)) => {
                RelationalInitialSchemaInstallationDenialKind::Request(
                    contracts::execution::denial_cause::RelationalExecutionDenialCause::AllocationFailed,
                )
            }
            Cause::Malformed
            | Cause::Source(_)
            | Cause::Missing
            | Cause::UnsupportedVersion(_)
            | Cause::Native(contracts::durability::data::recovery_stop::RelationalRecoveryStop::Durability(_))
            | Cause::Native(contracts::durability::data::recovery_stop::RelationalRecoveryStop::AdmissionImage(_))
            | Cause::Payload(_) => {
                RelationalInitialSchemaInstallationDenialKind::InitialInputsRejected
            }
        };
        Self::new(kind, format!("{cause:?}"))
    }
}

#[doc(hidden)]
pub trait RelationalInitialSchemaInstallationDenialOwnerPort: Sized {
    fn new(kind: RelationalInitialSchemaInstallationDenialKind, detail: impl Into<String>) -> Self;

    fn publication(
        cause: crate::durability::published_medium::RelationalInitialInputsPublicationStop,
    ) -> Self;
}

impl RelationalInitialSchemaInstallationDenialOwnerPort
    for RelationalInitialSchemaInstallationDenial
{
    fn new(kind: RelationalInitialSchemaInstallationDenialKind, detail: impl Into<String>) -> Self {
        Self::new(kind, detail)
    }

    fn publication(
        cause: crate::durability::published_medium::RelationalInitialInputsPublicationStop,
    ) -> Self {
        Self::publication(cause)
    }
}
