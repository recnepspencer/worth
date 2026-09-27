use super::{WorthUiDeclarationProjectionDenial, WorthUiSemanticHandoffEvidence};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthUiServiceDeclarationAdmissionCause {
    DuplicateIdentity,
    ConflictingFamilyPolicy,
    /// A smooth wheel horizon the runtime cannot honour.
    ScrollWheelHorizon(crate::declaration::UiScrollWheelBehaviorDenial),
    InvalidCommandIdentity,
    CommandNotRegistered,
    CommandShortcutMissing,
    CommandShortcutMismatch,
    CommandRouteMissing,
    CommandScopeMismatch,
    CommandScopeBindingUndeclared,
    CommandScopeBindingMismatch,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthUiSemanticHandoffPreparationStop {
    UnsupportedProtocol,
    AppearanceRoleRegistration(crate::capability::AppearanceRoleRegistrationDenial),
    AuthoredScrollRegion(crate::capability::UiAuthoredScrollRegionDenial),
    AuthoredLayout(crate::capability::UiAuthoredLayoutDenial),
    CapabilityResolution,
    RuntimeStructuralAdmission,
    DeclarationProjection,
    ComponentReference {
        declaration_index: usize,
        cause: crate::declaration::UiDeclarationComponentReferenceDenial,
    },
    AppearanceRoleAttachment {
        declaration_index: usize,
        cause: crate::declaration::UiAppearanceRoleAttachmentDenial,
    },
    IntentDeclaration,
    ServiceDeclaration {
        declaration_index: usize,
        cause: WorthUiServiceDeclarationAdmissionCause,
    },
    BindingAdmission,
    IdentitySeeding,
    CanonicalAssembly,
}

/// What the lowering behind a stop reported, so a denial names why the
/// source was refused and not only where.
#[derive(Clone, Eq, PartialEq)]
pub struct WorthUiSemanticHandoffPreparationCause(WorthUiSemanticHandoffPreparationReport);

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum WorthUiSemanticHandoffPreparationReport {
    IntentDeclaration(crate::declaration::WorthUiAuthoredIntentMaterialDenial),
    CapabilityResolution(crate::source::WorthUiResolutionReport),
    RuntimeStructuralAdmission(crate::source::WorthUiStructuralLegalityReport),
    DeclarationProjection(WorthUiDeclarationProjectionDenial),
    BindingAdmission(crate::source::WorthUiBindingSemanticsReport),
    IdentitySeeding(crate::source::WorthUiIdentitySeedingReport),
    CanonicalAssembly(crate::source::WorthUiArtifactAssemblyReport),
}

impl WorthUiSemanticHandoffPreparationCause {
    #[cfg(test)]
    pub(crate) fn report(&self) -> &WorthUiSemanticHandoffPreparationReport {
        &self.0
    }
}

impl std::fmt::Debug for WorthUiSemanticHandoffPreparationCause {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use WorthUiSemanticHandoffPreparationReport as Report;
        match &self.0 {
            Report::IntentDeclaration(denial) => denial.fmt(formatter),
            Report::CapabilityResolution(report) => report.fmt(formatter),
            Report::RuntimeStructuralAdmission(report) => report.fmt(formatter),
            Report::DeclarationProjection(denial) => denial.fmt(formatter),
            Report::BindingAdmission(report) => report.fmt(formatter),
            Report::IdentitySeeding(report) => report.fmt(formatter),
            Report::CanonicalAssembly(report) => report.fmt(formatter),
        }
    }
}

/// Typed runtime-owned stop after DSL sealing and before candidate mutation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthUiSemanticHandoffPreparationDenial {
    handoff: Box<WorthUiSemanticHandoffEvidence>,
    stop: WorthUiSemanticHandoffPreparationStop,
    cause: Option<Box<WorthUiSemanticHandoffPreparationCause>>,
}

impl WorthUiSemanticHandoffPreparationDenial {
    pub(super) fn new(
        handoff: WorthUiSemanticHandoffEvidence,
        stop: WorthUiSemanticHandoffPreparationStop,
    ) -> Self {
        Self {
            handoff: Box::new(handoff),
            stop,
            cause: None,
        }
    }

    pub(super) fn caused_by(mut self, report: WorthUiSemanticHandoffPreparationReport) -> Self {
        self.cause = Some(Box::new(WorthUiSemanticHandoffPreparationCause(report)));
        self
    }

    pub fn handoff(&self) -> &WorthUiSemanticHandoffEvidence {
        &self.handoff
    }

    pub fn stop(&self) -> WorthUiSemanticHandoffPreparationStop {
        self.stop
    }

    /// What the lowering behind the stop reported, for a stop that names only
    /// the phase that refused the source.
    pub fn cause(&self) -> Option<&WorthUiSemanticHandoffPreparationCause> {
        self.cause.as_deref()
    }
}
