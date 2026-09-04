use crate::capability::{UiIntentId, UiSemanticInteractionFamily};
use crate::declaration::UiCanonicalIntentDeclaration;
use crate::graph::UiGraphNodeIdentity;
use std::sync::Arc;

pub enum UiIntentRouteResolution {
    Product(UiResolvedProductIntentRoute),
    Confirmation(UiResolvedConfirmationIntentRoute),
}

pub struct UiResolvedProductIntentRoute {
    graph_node: UiGraphNodeIdentity,
    interaction: UiSemanticInteractionFamily,
    portal_declaration: Option<worth_ui_dsl::UiPortalDeclarationId>,
    definition_id: UiIntentId,
    declaration: Arc<UiCanonicalIntentDeclaration>,
    source: UiIntentProductInputSource,
    cost: crate::declaration::UiIntentRouteResolutionCost,
    evidence_reference: Option<worth_ui_inspection::UiIntentEvidenceReference>,
}

pub struct UiResolvedConfirmationIntentRoute {
    graph_node: UiGraphNodeIdentity,
    definition_id: UiIntentId,
    declaration: Arc<UiCanonicalIntentDeclaration>,
    source: crate::runtime::interaction::UiSemanticInteraction,
    cost: crate::declaration::UiIntentRouteResolutionCost,
    evidence_reference: Option<worth_ui_inspection::UiIntentEvidenceReference>,
}

pub(crate) struct UiResolvedProductIntentRouteInput {
    pub(crate) graph_node: UiGraphNodeIdentity,
    pub(crate) interaction: UiSemanticInteractionFamily,
    pub(crate) portal_declaration: Option<worth_ui_dsl::UiPortalDeclarationId>,
    pub(crate) definition_id: UiIntentId,
    pub(crate) declaration: Arc<UiCanonicalIntentDeclaration>,
    pub(crate) source: UiIntentProductInputSource,
    pub(crate) cost: crate::declaration::UiIntentRouteResolutionCost,
}

pub(crate) enum UiIntentProductInputSource {
    Mounted(crate::runtime::interaction::UiSemanticInteraction),
    Command {
        receipt: crate::runtime::UiCommandRouteReceipt,
        target: crate::runtime::interaction::UiPresentedInteractionTargetView,
    },
}

pub(crate) struct UiResolvedConfirmationIntentRouteInput {
    pub(crate) graph_node: UiGraphNodeIdentity,
    pub(crate) definition_id: UiIntentId,
    pub(crate) declaration: Arc<UiCanonicalIntentDeclaration>,
    pub(crate) source: crate::runtime::interaction::UiSemanticInteraction,
    pub(crate) cost: crate::declaration::UiIntentRouteResolutionCost,
}

impl UiResolvedProductIntentRoute {
    pub(crate) fn new(input: UiResolvedProductIntentRouteInput) -> Self {
        Self {
            graph_node: input.graph_node,
            interaction: input.interaction,
            portal_declaration: input.portal_declaration,
            definition_id: input.definition_id,
            declaration: input.declaration,
            source: input.source,
            cost: input.cost,
            evidence_reference: None,
        }
    }

    pub const fn graph_node(&self) -> UiGraphNodeIdentity {
        self.graph_node
    }

    pub const fn interaction(&self) -> UiSemanticInteractionFamily {
        self.interaction
    }

    pub const fn portal_declaration(&self) -> Option<worth_ui_dsl::UiPortalDeclarationId> {
        self.portal_declaration
    }

    pub const fn definition_id(&self) -> UiIntentId {
        self.definition_id
    }

    pub fn declaration_identity(&self) -> &str {
        self.declaration.identity().as_str()
    }

    pub const fn target(&self) -> crate::runtime::interaction::UiPresentedInteractionTargetView {
        self.source.target()
    }

    pub const fn mounted_interaction(
        &self,
    ) -> Option<&crate::runtime::interaction::UiSemanticInteraction> {
        self.source.mounted_interaction()
    }

    pub const fn is_command_route(&self) -> bool {
        self.source.is_command()
    }

    pub const fn cost(&self) -> crate::declaration::UiIntentRouteResolutionCost {
        self.cost
    }

    pub const fn evidence_reference(
        &self,
    ) -> Option<worth_ui_inspection::UiIntentEvidenceReference> {
        self.evidence_reference
    }

    pub(crate) fn command_evidence_reference(
        &self,
    ) -> Option<worth_ui_inspection::UiIntentEvidenceReference> {
        self.source
            .command_receipt()
            .and_then(crate::runtime::command_routing::UiCommandRouteReceipt::evidence_reference)
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        UiGraphNodeIdentity,
        Option<worth_ui_dsl::UiPortalDeclarationId>,
        Arc<UiCanonicalIntentDeclaration>,
        UiIntentProductInputSource,
        crate::declaration::UiIntentRouteResolutionCost,
        Option<worth_ui_inspection::UiIntentEvidenceReference>,
    ) {
        (
            self.graph_node,
            self.portal_declaration,
            self.declaration,
            self.source,
            self.cost,
            self.evidence_reference,
        )
    }
}

impl UiIntentProductInputSource {
    pub(crate) const fn mounted(
        interaction: crate::runtime::interaction::UiSemanticInteraction,
    ) -> Self {
        Self::Mounted(interaction)
    }

    pub(crate) const fn command(
        receipt: crate::runtime::UiCommandRouteReceipt,
        target: crate::runtime::interaction::UiPresentedInteractionTargetView,
    ) -> Self {
        Self::Command { receipt, target }
    }

    pub(crate) const fn generation(
        &self,
    ) -> &crate::runtime::WorthUiActiveApplicationGenerationIdentity {
        match self {
            Self::Mounted(interaction) => interaction.generation(),
            Self::Command { receipt, .. } => receipt.application(),
        }
    }

    pub(crate) const fn target(
        &self,
    ) -> crate::runtime::interaction::UiPresentedInteractionTargetView {
        match self {
            Self::Mounted(interaction) => interaction.target(),
            Self::Command { target, .. } => *target,
        }
    }

    pub(crate) const fn time_basis(
        &self,
    ) -> Option<worth_ui_host_contract::UiHostObservationTimeBasis> {
        match self {
            Self::Mounted(interaction) => Some(interaction.time_basis()),
            Self::Command { receipt, .. } => receipt.time_basis(),
        }
    }

    pub(crate) const fn mounted_interaction(
        &self,
    ) -> Option<&crate::runtime::interaction::UiSemanticInteraction> {
        match self {
            Self::Mounted(interaction) => Some(interaction),
            Self::Command { .. } => None,
        }
    }

    pub(crate) const fn command_receipt(&self) -> Option<&crate::runtime::UiCommandRouteReceipt> {
        match self {
            Self::Mounted(_) => None,
            Self::Command { receipt, .. } => Some(receipt),
        }
    }

    pub(crate) const fn is_command(&self) -> bool {
        matches!(self, Self::Command { .. })
    }
}

impl UiResolvedConfirmationIntentRoute {
    pub(crate) fn new(input: UiResolvedConfirmationIntentRouteInput) -> Self {
        Self {
            graph_node: input.graph_node,
            definition_id: input.definition_id,
            declaration: input.declaration,
            source: input.source,
            cost: input.cost,
            evidence_reference: None,
        }
    }

    pub const fn graph_node(&self) -> UiGraphNodeIdentity {
        self.graph_node
    }

    pub const fn definition_id(&self) -> UiIntentId {
        self.definition_id
    }

    pub fn declaration_identity(&self) -> &str {
        self.declaration.identity().as_str()
    }

    pub const fn source(&self) -> &crate::runtime::interaction::UiSemanticInteraction {
        &self.source
    }

    pub const fn cost(&self) -> crate::declaration::UiIntentRouteResolutionCost {
        self.cost
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        UiGraphNodeIdentity,
        UiIntentId,
        Arc<UiCanonicalIntentDeclaration>,
        crate::runtime::interaction::UiSemanticInteraction,
        crate::declaration::UiIntentRouteResolutionCost,
        Option<worth_ui_inspection::UiIntentEvidenceReference>,
    ) {
        (
            self.graph_node,
            self.definition_id,
            self.declaration,
            self.source,
            self.cost,
            self.evidence_reference,
        )
    }
}

impl UiIntentRouteResolution {
    pub(crate) fn command_evidence_reference(
        &self,
    ) -> Option<worth_ui_inspection::UiIntentEvidenceReference> {
        match self {
            Self::Product(route) => route.command_evidence_reference(),
            Self::Confirmation(_) => None,
        }
    }

    pub(crate) fn with_evidence_reference(
        mut self,
        reference: Option<worth_ui_inspection::UiIntentEvidenceReference>,
    ) -> Self {
        match &mut self {
            Self::Product(route) => route.evidence_reference = reference,
            Self::Confirmation(route) => route.evidence_reference = reference,
        }
        self
    }
}
