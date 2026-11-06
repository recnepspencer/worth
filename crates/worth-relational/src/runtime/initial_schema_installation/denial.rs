use crate::schema::data::SchemaRegistryError;
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
    Request(crate::execution::RelationalExecutionDenialCause),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelationalInitialSchemaInstallationDenial {
    kind: RelationalInitialSchemaInstallationDenialKind,
    detail: String,
    publication: Option<crate::durability::medium::RelationalInitialInputsPublicationStop>,
}

impl RelationalInitialSchemaInstallationDenial {
    pub(super) fn new(
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
    ) -> Option<&crate::durability::medium::RelationalInitialInputsPublicationStop> {
        self.publication.as_ref()
    }
    pub(super) fn publication(
        cause: crate::durability::medium::RelationalInitialInputsPublicationStop,
    ) -> Self {
        use crate::durability::medium::RelationalInitialInputsPublicationStop as Stop;
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
                crate::durability::medium::RelationalDurableOwnedReadRefusal::Request(cause),
            )
            | Cause::Native(crate::durability::data::RelationalRecoveryStop::Request(cause)) => {
                RelationalInitialSchemaInstallationDenialKind::Request(*cause)
            }
            Cause::Native(crate::durability::data::RelationalRecoveryStop::Content(cause)) => {
                RelationalInitialSchemaInstallationDenialKind::Request(
                    (*cause).into_execution_cause(),
                )
            }
            Cause::Native(crate::durability::data::RelationalRecoveryStop::Allocation(_)) => {
                RelationalInitialSchemaInstallationDenialKind::Request(
                    crate::execution::RelationalExecutionDenialCause::AllocationFailed,
                )
            }
            Cause::Malformed
            | Cause::Source(_)
            | Cause::Missing
            | Cause::UnsupportedVersion(_)
            | Cause::Native(crate::durability::data::RelationalRecoveryStop::Durability(_))
            | Cause::Native(crate::durability::data::RelationalRecoveryStop::AdmissionImage(_))
            | Cause::Payload(_) => {
                RelationalInitialSchemaInstallationDenialKind::InitialInputsRejected
            }
        };
        Self::new(kind, format!("{cause:?}"))
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

impl RelationalInitialSchemaInstallationDenial {
    pub(super) fn original_capture(
        cause: crate::runtime::OriginalBranchSchemaInputsDenial,
    ) -> Self {
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
        Self::new(kind, "original initialization capture refused")
    }
}
