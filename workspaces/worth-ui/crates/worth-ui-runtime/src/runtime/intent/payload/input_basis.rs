#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UiIntentPayloadProjectionCost {
    declared_fields: usize,
    query_inputs_read: usize,
    application_inputs_read: usize,
    admitted_utf8_bytes: usize,
}

pub struct UiIntentInputBasisReceipt {
    generation: crate::runtime::WorthUiActiveApplicationGenerationIdentity,
    publication_frame: worth_ui_host_contract::UiMountedFrameIdentity,
    target: crate::runtime::interaction::UiPresentedInteractionTargetView,
    portal_declaration: Option<worth_ui_dsl::UiPortalDeclarationId>,
    route_resolution: crate::declaration::UiIntentRouteResolutionCost,
    cost: UiIntentPayloadProjectionCost,
    owner_revisions: Box<[UiIntentInputOwnerRevision]>,
    evidence_reference: Option<worth_ui_inspection::UiIntentEvidenceReference>,
}

pub(crate) struct UiIntentInputBasis {
    receipt: UiIntentInputBasisReceipt,
    source: super::super::routing::UiIntentProductInputSource,
    query_inputs: Box<[worth_ui_query_binding::UiProjectionInputFactReference]>,
    application_inputs: Box<[super::UiIntentApplicationInputReference]>,
    operability: super::super::operability::UiIntentOperabilityBasis,
}

pub(crate) struct UiIntentInputBasisInput {
    pub(crate) generation: crate::runtime::WorthUiActiveApplicationGenerationIdentity,
    pub(crate) publication_frame: worth_ui_host_contract::UiMountedFrameIdentity,
    pub(crate) target: crate::runtime::interaction::UiPresentedInteractionTargetView,
    pub(crate) portal_declaration: Option<worth_ui_dsl::UiPortalDeclarationId>,
    pub(crate) route_resolution: crate::declaration::UiIntentRouteResolutionCost,
    pub(crate) source: super::super::routing::UiIntentProductInputSource,
    pub(crate) query_inputs: Vec<worth_ui_query_binding::UiProjectionInputFactReference>,
    pub(crate) application_inputs: Vec<super::UiIntentApplicationInputReference>,
    pub(crate) owner_revisions: Vec<UiIntentInputOwnerRevision>,
    pub(crate) cost: UiIntentPayloadProjectionCost,
    pub(crate) operability: super::super::operability::UiIntentOperabilityBasis,
    pub(crate) evidence_reference: Option<worth_ui_inspection::UiIntentEvidenceReference>,
}

pub(crate) struct UiIntentInputBasisMaterial {
    pub(crate) source: super::super::routing::UiIntentProductInputSource,
    pub(crate) portal_declaration: Option<worth_ui_dsl::UiPortalDeclarationId>,
    pub(crate) query_inputs: Vec<worth_ui_query_binding::UiProjectionInputFactReference>,
    pub(crate) application_inputs: Vec<super::UiIntentApplicationInputReference>,
    pub(crate) owner_revisions: Vec<UiIntentInputOwnerRevision>,
    pub(crate) route_resolution: crate::declaration::UiIntentRouteResolutionCost,
    pub(crate) cost: UiIntentPayloadProjectionCost,
    pub(crate) operability: super::super::operability::UiIntentOperabilityBasis,
    pub(crate) evidence_reference: Option<worth_ui_inspection::UiIntentEvidenceReference>,
}

impl UiIntentInputBasis {
    pub(crate) fn seal(input: UiIntentInputBasisInput) -> Self {
        Self {
            receipt: UiIntentInputBasisReceipt {
                generation: input.generation,
                publication_frame: input.publication_frame,
                target: input.target,
                portal_declaration: input.portal_declaration,
                route_resolution: input.route_resolution,
                cost: input.cost,
                owner_revisions: input.owner_revisions.into_boxed_slice(),
                evidence_reference: input.evidence_reference,
            },
            source: input.source,
            query_inputs: input.query_inputs.into_boxed_slice(),
            application_inputs: input.application_inputs.into_boxed_slice(),
            operability: input.operability,
        }
    }

    pub(crate) const fn receipt(&self) -> &UiIntentInputBasisReceipt {
        &self.receipt
    }

    pub(crate) fn retained_owner_reference_count(&self) -> usize {
        let _ = &self.source;
        1 + self.query_inputs.len() + self.application_inputs.len()
    }

    pub(crate) const fn operability(&self) -> &super::super::operability::UiIntentOperabilityBasis {
        &self.operability
    }

    pub(crate) fn interaction_time_basis(
        &self,
    ) -> worth_ui_host_contract::UiHostObservationTimeBasis {
        self.source
            .time_basis()
            .expect("resolved command routes carry an exact observation time basis")
    }

    pub(crate) fn selection_option(
        &self,
    ) -> Option<&worth_ui_query_binding::UiProjectionOptionReference> {
        match self.source.mounted_interaction() {
            Some(crate::runtime::interaction::UiSemanticInteraction::SelectionCommit(
                selection,
            )) => Some(selection.option()),
            _ => None,
        }
    }

    pub(crate) const fn command_route_receipt(
        &self,
    ) -> Option<&crate::runtime::UiCommandRouteReceipt> {
        self.source.command_receipt()
    }

    pub(crate) fn payload_inputs_are_current(
        &self,
        mounted: &crate::mounting::WorthUiMountedSessionState,
        application_facts: &super::UiIntentApplicationFactState,
        generation: &crate::runtime::WorthUiActiveApplicationGenerationIdentity,
    ) -> bool {
        self.query_inputs.iter().all(|expected| {
            mounted
                .current_projection_input(expected.revision().slot())
                .as_ref()
                == Some(expected)
        }) && self
            .application_inputs
            .iter()
            .all(|expected| application_facts.is_current_reference(expected, generation))
    }
}

impl UiIntentInputBasisReceipt {
    pub const fn generation(&self) -> &crate::runtime::WorthUiActiveApplicationGenerationIdentity {
        &self.generation
    }

    pub const fn publication_frame(&self) -> worth_ui_host_contract::UiMountedFrameIdentity {
        self.publication_frame
    }

    pub const fn target(&self) -> crate::runtime::interaction::UiPresentedInteractionTargetView {
        self.target
    }

    pub const fn portal_declaration(&self) -> Option<worth_ui_dsl::UiPortalDeclarationId> {
        self.portal_declaration
    }

    pub const fn cost(&self) -> UiIntentPayloadProjectionCost {
        self.cost
    }

    pub const fn route_resolution_cost(&self) -> crate::declaration::UiIntentRouteResolutionCost {
        self.route_resolution
    }

    pub fn owner_revisions(&self) -> &[UiIntentInputOwnerRevision] {
        &self.owner_revisions
    }

    pub const fn evidence_reference(
        &self,
    ) -> Option<worth_ui_inspection::UiIntentEvidenceReference> {
        self.evidence_reference
    }
}

impl UiIntentPayloadProjectionCost {
    pub(crate) fn record_field(&mut self) {
        self.declared_fields = next(self.declared_fields);
    }

    pub(crate) fn record_query_input(&mut self) {
        self.query_inputs_read = next(self.query_inputs_read);
    }

    pub(crate) fn record_application_input(&mut self) {
        self.application_inputs_read = next(self.application_inputs_read);
    }

    pub(crate) fn record_utf8_bytes(&mut self, bytes: usize) {
        self.admitted_utf8_bytes = self
            .admitted_utf8_bytes
            .checked_add(bytes)
            .expect("bounded payload byte accounting exhausted");
    }

    pub const fn declared_fields(self) -> usize {
        self.declared_fields
    }

    pub const fn query_inputs_read(self) -> usize {
        self.query_inputs_read
    }

    pub const fn application_inputs_read(self) -> usize {
        self.application_inputs_read
    }

    pub const fn admitted_utf8_bytes(self) -> usize {
        self.admitted_utf8_bytes
    }
}

fn next(value: usize) -> usize {
    value
        .checked_add(1)
        .expect("bounded payload field accounting exhausted")
}
mod owner_revision;
mod view;

pub use owner_revision::{
    UiIntentApplicationFactRevision, UiIntentDraftInputRevision, UiIntentInputOwnerRevision,
    UiIntentQueryInputRevision,
};
pub(crate) use view::UiIntentInputBasisView;
