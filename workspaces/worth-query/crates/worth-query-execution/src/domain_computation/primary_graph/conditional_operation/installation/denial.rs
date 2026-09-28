/// Why installing, reinstalling, or admitting a conditional runtime was
/// refused.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryConditionalRuntimeInstallationDenialKind {
    /// The application schema or runtime publication was rejected.
    PrimaryGraphPublication,
    /// A binding does not belong to this installation.
    ForeignBinding,
    /// The same binding was installed twice.
    DuplicateBinding,
    /// Not every expected conditional binding was installed.
    IncompleteBindingInventory,
    /// The Runtime Bridge rejected the conditional runtime.
    BridgeRejected,
    /// The principal for reconstruction could not be resolved.
    ReconstructionPrincipal,
    /// The scope for reconstruction could not be resolved.
    ReconstructionScope,
    /// The reconstruction query was refused.
    ReconstructionQuery,
    /// The installed limit on concurrently active snapshots was reached.
    ActiveSnapshotCapacityExhausted { maximum_active_snapshots: usize },
    /// No capacity remains to retain a basis.
    RetentionCapacityExhausted,
    /// The runtime ran out of basis-retention identities.
    RetentionIdentityExhausted,
    /// The runtime ran out of snapshot identities.
    SnapshotIdentityExhausted,
    /// Projecting a reconstruction row into a temporal intent failed.
    ReconstructionProjection,
    /// A reconstructed temporal intent was rejected.
    ReconstructionIntent,
    /// The installation changed; build a new runtime with fresh bindings instead
    /// of reinstalling.
    RebindRequired,
}

/// Refusal to install, reinstall, or admit a conditional runtime.
///
/// [`Self::kind`] says why and [`Self::subject`] names what was refused.
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
