use super::WorthUiSemanticHandoffEvidence;

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

/// Typed runtime-owned stop after DSL sealing and before candidate mutation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthUiSemanticHandoffPreparationDenial {
    handoff: Box<WorthUiSemanticHandoffEvidence>,
    stop: WorthUiSemanticHandoffPreparationStop,
}

impl WorthUiSemanticHandoffPreparationDenial {
    pub(super) fn new(
        handoff: WorthUiSemanticHandoffEvidence,
        stop: WorthUiSemanticHandoffPreparationStop,
    ) -> Self {
        Self {
            handoff: Box::new(handoff),
            stop,
        }
    }

    pub fn handoff(&self) -> &WorthUiSemanticHandoffEvidence {
        &self.handoff
    }

    pub fn stop(&self) -> WorthUiSemanticHandoffPreparationStop {
        self.stop
    }
}
