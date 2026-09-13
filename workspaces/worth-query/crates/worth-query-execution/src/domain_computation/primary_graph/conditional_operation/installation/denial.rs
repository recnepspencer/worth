#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryConditionalRuntimeInstallationDenialKind {
    PrimaryGraphPublication,
    ForeignBinding,
    DuplicateBinding,
    IncompleteBindingInventory,
    BridgeRejected,
    ReconstructionPrincipal,
    ReconstructionScope,
    ReconstructionQuery,
    ActiveSnapshotCapacityExhausted { maximum_active_snapshots: usize },
    RetentionCapacityExhausted,
    RetentionIdentityExhausted,
    SnapshotIdentityExhausted,
    ReconstructionProjection,
    ReconstructionIntent,
    RebindRequired,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryConditionalRuntimeInstallationDenial {
    kind: WorthQueryConditionalRuntimeInstallationDenialKind,
    subject: String,
}

impl WorthQueryConditionalRuntimeInstallationDenial {
    pub(in crate::domain_computation::primary_graph) fn new(
        kind: WorthQueryConditionalRuntimeInstallationDenialKind,
        subject: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            subject: subject.into(),
        }
    }

    pub fn kind(&self) -> WorthQueryConditionalRuntimeInstallationDenialKind {
        self.kind
    }

    pub fn subject(&self) -> &str {
        &self.subject
    }
}

pub(super) fn foreign_binding_denial(
    subject: impl Into<String>,
) -> WorthQueryConditionalRuntimeInstallationDenial {
    WorthQueryConditionalRuntimeInstallationDenial::new(
        WorthQueryConditionalRuntimeInstallationDenialKind::ForeignBinding,
        subject,
    )
}
