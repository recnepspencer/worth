use crate::capability::MosaicSizingContractId;
use crate::declaration::{
    UiAspectContract, UiDeclarationContainmentIntent, UiDeclarationFamily, UiDeclarationFamilyKind,
    UiDeclarationIdentity, UiDeclarationOrderingGuarantee, UiDeclarationPlanningOperatorKind,
    UiDeclarationRepetitionPosture, UiDeclarationSlotParticipationIntent,
    UiDeclarationStructuralDigest, UiDeclarationStructuralRole, UiDeclaredAspectPayload,
    UiDeclaredHostCapabilityPosture, UiDeclaredMeasurementPolicyPosture, UiDeclaredPostureContract,
    UiDeclaredPostureLane, UiDeclaredPosturePayload, UiDeclaredQueryBindingPosture,
    UiDeclaredServiceUsagePosture, UiDeclaredTouchMeaningPosture, UiStructuralDeclarationPayload,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiDeclarationGraphHandoff {
    identity: UiDeclarationIdentity,
    authored_provenance_digest: u64,
    structural: UiStructuralDeclarationPayload,
    aspect_contract: UiDeclaredAspectPayload,
    declared_posture: UiDeclaredPosturePayload,
    component_reference: Option<crate::capability::ComponentId>,
    appearance_role_attachment: Option<crate::declaration::UiAppearanceRoleAttachment>,
}

impl UiDeclarationGraphHandoff {
    pub(crate) fn new(
        identity: UiDeclarationIdentity,
        authored_provenance_digest: u64,
        structural: UiStructuralDeclarationPayload,
        aspect_contract: UiDeclaredAspectPayload,
        declared_posture: UiDeclaredPosturePayload,
        component_reference: Option<crate::capability::ComponentId>,
        appearance_role_attachment: Option<crate::declaration::UiAppearanceRoleAttachment>,
    ) -> Self {
        Self {
            identity,
            authored_provenance_digest,
            structural,
            aspect_contract,
            declared_posture,
            component_reference,
            appearance_role_attachment,
        }
    }

    pub fn identity(&self) -> &UiDeclarationIdentity {
        &self.identity
    }

    pub fn authored_provenance_digest(&self) -> u64 {
        self.authored_provenance_digest
    }

    pub fn family(&self) -> &UiDeclarationFamily {
        self.structural.family()
    }

    pub const fn family_kind(&self) -> UiDeclarationFamilyKind {
        self.structural.family_kind()
    }

    pub const fn structural_digest(&self) -> UiDeclarationStructuralDigest {
        self.structural.structural_digest()
    }

    pub const fn role(&self) -> UiDeclarationStructuralRole {
        self.structural.role()
    }

    pub const fn operator_kind(&self) -> UiDeclarationPlanningOperatorKind {
        self.structural.operator_kind()
    }

    pub fn mosaic_sizing_contract_id(&self) -> Option<&MosaicSizingContractId> {
        self.structural.mosaic_sizing_contract_id()
    }

    pub fn containment_intent(&self) -> &UiDeclarationContainmentIntent {
        self.structural.containment_intent()
    }

    pub fn slot_participation_intent(&self) -> &UiDeclarationSlotParticipationIntent {
        self.structural.slot_participation_intent()
    }

    pub const fn ordering_guarantee(&self) -> UiDeclarationOrderingGuarantee {
        self.structural.ordering_guarantee()
    }

    pub const fn repetition_posture(&self) -> UiDeclarationRepetitionPosture {
        self.structural.repetition_posture()
    }

    pub const fn structural_semantics(
        &self,
    ) -> &crate::declaration::UiDeclarationStructuralSemantics {
        self.structural.semantics()
    }

    pub const fn aspect_contract(&self) -> &UiAspectContract {
        self.aspect_contract.contract()
    }

    pub const fn declared_posture(&self) -> &UiDeclaredPostureContract {
        self.declared_posture.contract()
    }

    pub const fn query_binding(&self) -> &UiDeclaredPostureLane<UiDeclaredQueryBindingPosture> {
        self.declared_posture.query_binding()
    }

    pub const fn service_usage(&self) -> &UiDeclaredPostureLane<UiDeclaredServiceUsagePosture> {
        self.declared_posture.service_usage()
    }

    pub const fn touch_meaning(&self) -> &UiDeclaredPostureLane<UiDeclaredTouchMeaningPosture> {
        self.declared_posture.touch_meaning()
    }

    pub const fn measurement_policy(
        &self,
    ) -> &UiDeclaredPostureLane<UiDeclaredMeasurementPolicyPosture> {
        self.declared_posture.measurement_policy()
    }

    pub const fn host_capability(&self) -> &UiDeclaredPostureLane<UiDeclaredHostCapabilityPosture> {
        self.declared_posture.host_capability()
    }

    pub(crate) const fn appearance_role_attachment(
        &self,
    ) -> Option<&crate::declaration::UiAppearanceRoleAttachment> {
        self.appearance_role_attachment.as_ref()
    }

    pub(crate) const fn component_reference(&self) -> Option<&crate::capability::ComponentId> {
        self.component_reference.as_ref()
    }
}
