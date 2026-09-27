use crate::capability::{
    AdmittedCapability, ComponentDescriptor, ComponentId, FrozenThemeTokenEntry, SurfaceDescriptor,
    SurfaceId, ThemeTokenId,
};
use crate::source::{
    WorthUiArtifactHandle, WorthUiArtifactIdentitySeed, WorthUiBoundSurfaceSemantics,
    WorthUiBoundThemeTokenSemantics, WorthUiBoundViewBindingReference,
    WorthUiDurableStateEligibility, WorthUiMosaicStructureFacts,
};
use worth_ui_dsl::WorthUiArtifactInputReference;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum WorthUiArtifactNode {
    Import(WorthUiArtifactImportNode),
    Component(WorthUiArtifactComponentNode),
    Surface(Box<WorthUiArtifactSurfaceNode>),
    Binding(WorthUiArtifactBindingNode),
    Token(WorthUiArtifactThemeTokenNode),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct WorthUiArtifactImportNode {
    handle: WorthUiArtifactHandle,
    target: WorthUiArtifactInputReference,
    authored_provenance_digest: u64,
    identity_seed: WorthUiArtifactIdentitySeed,
    durable_state_eligibility: WorthUiDurableStateEligibility,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct WorthUiArtifactComponentNode {
    handle: WorthUiArtifactHandle,
    component: AdmittedCapability<ComponentId>,
    descriptor: ComponentDescriptor,
    structure: WorthUiMosaicStructureFacts,
    authored_provenance_digest: u64,
    identity_seed: WorthUiArtifactIdentitySeed,
    durable_state_eligibility: WorthUiDurableStateEligibility,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct WorthUiArtifactSurfaceNode {
    handle: WorthUiArtifactHandle,
    surface: AdmittedCapability<SurfaceId>,
    descriptor: SurfaceDescriptor,
    structure: WorthUiMosaicStructureFacts,
    semantics: WorthUiBoundSurfaceSemantics,
    authored_provenance_digest: u64,
    identity_seed: WorthUiArtifactIdentitySeed,
    durable_state_eligibility: WorthUiDurableStateEligibility,
}

pub(crate) struct WorthUiArtifactSurfaceNodeInput {
    pub handle: WorthUiArtifactHandle,
    pub surface: AdmittedCapability<SurfaceId>,
    pub descriptor: SurfaceDescriptor,
    pub structure: WorthUiMosaicStructureFacts,
    pub semantics: WorthUiBoundSurfaceSemantics,
    pub authored_provenance_digest: u64,
    pub identity_seed: WorthUiArtifactIdentitySeed,
    pub durable_state_eligibility: WorthUiDurableStateEligibility,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct WorthUiArtifactBindingNode {
    handle: WorthUiArtifactHandle,
    view_binding_reference: WorthUiBoundViewBindingReference,
    structure: WorthUiMosaicStructureFacts,
    authored_provenance_digest: u64,
    identity_seed: WorthUiArtifactIdentitySeed,
    durable_state_eligibility: WorthUiDurableStateEligibility,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct WorthUiArtifactThemeTokenNode {
    handle: WorthUiArtifactHandle,
    theme_token: AdmittedCapability<ThemeTokenId>,
    entry: FrozenThemeTokenEntry,
    semantics: WorthUiBoundThemeTokenSemantics,
    authored_provenance_digest: u64,
    identity_seed: WorthUiArtifactIdentitySeed,
    durable_state_eligibility: WorthUiDurableStateEligibility,
}

macro_rules! artifact_node_common_accessors {
    ($name:ident) => {
        impl $name {
            pub(crate) fn handle(&self) -> &WorthUiArtifactHandle {
                &self.handle
            }

            pub(crate) fn identity_seed(&self) -> &WorthUiArtifactIdentitySeed {
                &self.identity_seed
            }

            pub(crate) fn authored_provenance_digest(&self) -> u64 {
                self.authored_provenance_digest
            }

            pub(crate) fn durable_state_eligibility(&self) -> &WorthUiDurableStateEligibility {
                &self.durable_state_eligibility
            }
        }
    };
}

artifact_node_common_accessors!(WorthUiArtifactImportNode);
artifact_node_common_accessors!(WorthUiArtifactComponentNode);
artifact_node_common_accessors!(WorthUiArtifactSurfaceNode);
artifact_node_common_accessors!(WorthUiArtifactBindingNode);
artifact_node_common_accessors!(WorthUiArtifactThemeTokenNode);

impl WorthUiArtifactNode {
    pub(crate) fn authored_declaration_identity(&self) -> String {
        match self {
            Self::Import(node) => format!("import:{}", node.target().authored_text()),
            Self::Component(node) => format!("component:{}", node.component().id().as_str()),
            Self::Surface(node) => format!("surface:{}", node.surface().id().as_str()),
            Self::Binding(node) => format!(
                "binding:{}",
                node.view_binding_reference().view_binding().id().as_str()
            ),
            Self::Token(node) => format!("token:{}", node.theme_token().id().as_str()),
        }
    }

    pub(crate) fn component_capability_identity(&self) -> Option<&str> {
        match self {
            Self::Component(node) => Some(node.component().id().as_str()),
            Self::Import(_) | Self::Surface(_) | Self::Binding(_) | Self::Token(_) => None,
        }
    }

    pub(crate) fn theme_token_capability_identity(&self) -> Option<&str> {
        match self {
            Self::Token(node) => Some(node.theme_token().id().as_str()),
            Self::Import(_) | Self::Component(_) | Self::Surface(_) | Self::Binding(_) => None,
        }
    }

    pub(crate) fn has_same_semantic_meaning_ignoring_location(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Import(left), Self::Import(right)) => left.target == right.target,
            (Self::Component(left), Self::Component(right)) => {
                left.component == right.component
                    && left.descriptor == right.descriptor
                    && left.structure == right.structure
            }
            (Self::Surface(left), Self::Surface(right)) => {
                left.surface == right.surface
                    && left.descriptor == right.descriptor
                    && left.structure == right.structure
                    && left.semantics == right.semantics
            }
            (Self::Binding(left), Self::Binding(right)) => {
                left.view_binding_reference == right.view_binding_reference
                    && left.structure == right.structure
            }
            (Self::Token(left), Self::Token(right)) => {
                left.theme_token == right.theme_token
                    && left.entry == right.entry
                    && left.semantics == right.semantics
            }
            _ => false,
        }
    }

    pub(crate) fn handle(&self) -> &WorthUiArtifactHandle {
        match self {
            Self::Import(node) => node.handle(),
            Self::Component(node) => node.handle(),
            Self::Surface(node) => node.handle(),
            Self::Binding(node) => node.handle(),
            Self::Token(node) => node.handle(),
        }
    }

    pub(crate) fn authored_provenance_digest(&self) -> u64 {
        match self {
            Self::Import(node) => node.authored_provenance_digest(),
            Self::Component(node) => node.authored_provenance_digest(),
            Self::Surface(node) => node.authored_provenance_digest(),
            Self::Binding(node) => node.authored_provenance_digest(),
            Self::Token(node) => node.authored_provenance_digest(),
        }
    }

    pub(crate) fn identity_seed(&self) -> &WorthUiArtifactIdentitySeed {
        match self {
            Self::Import(node) => node.identity_seed(),
            Self::Component(node) => node.identity_seed(),
            Self::Surface(node) => node.identity_seed(),
            Self::Binding(node) => node.identity_seed(),
            Self::Token(node) => node.identity_seed(),
        }
    }
}

impl WorthUiArtifactImportNode {
    pub(crate) fn new(
        handle: WorthUiArtifactHandle,
        target: WorthUiArtifactInputReference,
        authored_provenance_digest: u64,
        identity_seed: WorthUiArtifactIdentitySeed,
        durable_state_eligibility: WorthUiDurableStateEligibility,
    ) -> Self {
        Self {
            handle,
            target,
            authored_provenance_digest,
            identity_seed,
            durable_state_eligibility,
        }
    }

    pub(crate) fn target(&self) -> &worth_ui_dsl::WorthUiArtifactInputReference {
        &self.target
    }
}

impl WorthUiArtifactComponentNode {
    pub(crate) fn new(
        handle: WorthUiArtifactHandle,
        component: AdmittedCapability<ComponentId>,
        descriptor: ComponentDescriptor,
        structure: WorthUiMosaicStructureFacts,
        authored_provenance_digest: u64,
        identity_seed: WorthUiArtifactIdentitySeed,
        durable_state_eligibility: WorthUiDurableStateEligibility,
    ) -> Self {
        Self {
            handle,
            component,
            descriptor,
            structure,
            authored_provenance_digest,
            identity_seed,
            durable_state_eligibility,
        }
    }

    pub(crate) fn component(&self) -> &AdmittedCapability<ComponentId> {
        &self.component
    }

    pub(crate) fn descriptor(&self) -> &ComponentDescriptor {
        &self.descriptor
    }

    pub(crate) fn structure(&self) -> &WorthUiMosaicStructureFacts {
        &self.structure
    }
}

impl WorthUiArtifactSurfaceNode {
    pub(crate) fn new(input: WorthUiArtifactSurfaceNodeInput) -> Self {
        let WorthUiArtifactSurfaceNodeInput {
            handle,
            surface,
            descriptor,
            structure,
            semantics,
            authored_provenance_digest,
            identity_seed,
            durable_state_eligibility,
        } = input;
        Self {
            handle,
            surface,
            descriptor,
            structure,
            semantics,
            authored_provenance_digest,
            identity_seed,
            durable_state_eligibility,
        }
    }

    pub(crate) fn surface(&self) -> &AdmittedCapability<SurfaceId> {
        &self.surface
    }

    pub(crate) fn descriptor(&self) -> &SurfaceDescriptor {
        &self.descriptor
    }

    pub(crate) fn structure(&self) -> &WorthUiMosaicStructureFacts {
        &self.structure
    }

    pub(crate) fn semantics(&self) -> &WorthUiBoundSurfaceSemantics {
        &self.semantics
    }
}

impl WorthUiArtifactBindingNode {
    pub(crate) fn new(
        handle: WorthUiArtifactHandle,
        view_binding_reference: WorthUiBoundViewBindingReference,
        structure: WorthUiMosaicStructureFacts,
        authored_provenance_digest: u64,
        identity_seed: WorthUiArtifactIdentitySeed,
        durable_state_eligibility: WorthUiDurableStateEligibility,
    ) -> Self {
        Self {
            handle,
            view_binding_reference,
            structure,
            authored_provenance_digest,
            identity_seed,
            durable_state_eligibility,
        }
    }

    pub(crate) fn view_binding_reference(&self) -> &WorthUiBoundViewBindingReference {
        &self.view_binding_reference
    }

    pub(crate) fn structure(&self) -> &WorthUiMosaicStructureFacts {
        &self.structure
    }
}

impl WorthUiArtifactThemeTokenNode {
    pub(crate) fn new(
        handle: WorthUiArtifactHandle,
        theme_token: AdmittedCapability<ThemeTokenId>,
        entry: FrozenThemeTokenEntry,
        semantics: WorthUiBoundThemeTokenSemantics,
        authored_provenance_digest: u64,
        identity_seed: WorthUiArtifactIdentitySeed,
        durable_state_eligibility: WorthUiDurableStateEligibility,
    ) -> Self {
        Self {
            handle,
            theme_token,
            entry,
            semantics,
            authored_provenance_digest,
            identity_seed,
            durable_state_eligibility,
        }
    }

    pub(crate) fn theme_token(&self) -> &AdmittedCapability<ThemeTokenId> {
        &self.theme_token
    }

    pub(crate) fn entry(&self) -> &FrozenThemeTokenEntry {
        &self.entry
    }

    pub(crate) fn semantics(&self) -> &WorthUiBoundThemeTokenSemantics {
        &self.semantics
    }
}
