use super::{admit_graph_handoffs, UiGraphParticipationSeed};
use crate::declaration::{
    UiAspectContract, UiDeclarationIdentity, UiDeclarationOrderingGuarantee,
    UiDeclarationPlanningOperatorKind, UiDeclarationRepetitionPosture,
    UiDeclarationSlotParticipationIntent, UiDeclarationStructuralDigest,
    UiDeclarationStructuralRole, UiDeclaredMeasurementBasisSource,
    UiDeclaredMeasurementConstraintModifier,
};
use crate::graph::{
    UiGraphAttachmentPosture, UiGraphContainmentClaim, UiGraphInstantiationDenial,
    UiGraphMountEligibilitySeed, UiGraphParentResolutionClaim, UiRepeatedInstanceBasis,
    UiRepeatedInstanceBasisDenial, UiRuntimeInstanceBasisAdmission,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiGraphInstantiationPlan {
    node_entries: Vec<UiGraphNodeInstantiationEntry>,
    local_denials: Vec<UiGraphInstantiationLocalDenial>,
}

impl UiGraphInstantiationPlan {
    pub fn admit_handoffs(
        handoffs: &[crate::declaration::UiDeclarationGraphHandoff],
        runtime_basis_admissions: &[UiRuntimeInstanceBasisAdmission],
    ) -> Result<Self, UiGraphInstantiationDenial> {
        admit_graph_handoffs(handoffs, runtime_basis_admissions)
    }

    pub(crate) fn new(
        node_entries: Vec<UiGraphNodeInstantiationEntry>,
        local_denials: Vec<UiGraphInstantiationLocalDenial>,
    ) -> Self {
        Self {
            node_entries,
            local_denials,
        }
    }

    pub fn node_entries(&self) -> &[UiGraphNodeInstantiationEntry] {
        &self.node_entries
    }

    pub fn local_denials(&self) -> &[UiGraphInstantiationLocalDenial] {
        &self.local_denials
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiGraphNodeInstantiationEntry {
    declaration_identity: UiDeclarationIdentity,
    authored_provenance_digest: u64,
    measurement_constraint_modifier: Option<UiDeclaredMeasurementConstraintModifier>,
    measurement_basis_source: Option<UiDeclaredMeasurementBasisSource>,
    aspect_contract: UiAspectContract,
    component_reference: Option<crate::capability::ComponentId>,
    appearance_role_attachment: Option<crate::declaration::UiAppearanceRoleAttachment>,
    repeated_instance_basis: UiRepeatedInstanceBasis,
    topology_seed: UiGraphTopologySeed,
    participation_seed: UiGraphParticipationSeed,
    attachment_posture: UiGraphAttachmentPosture,
    mount_eligibility_seed: UiGraphMountEligibilitySeed,
    core_index_contribution_seed: UiGraphCoreIndexContributionSeed,
}

pub(crate) struct UiGraphNodeInstantiationInput {
    pub(crate) declaration_identity: UiDeclarationIdentity,
    pub(crate) authored_provenance_digest: u64,
    pub(crate) measurement_constraint_modifier: Option<UiDeclaredMeasurementConstraintModifier>,
    pub(crate) measurement_basis_source: Option<UiDeclaredMeasurementBasisSource>,
    pub(crate) aspect_contract: UiAspectContract,
    pub(crate) component_reference: Option<crate::capability::ComponentId>,
    pub(crate) appearance_role_attachment: Option<crate::declaration::UiAppearanceRoleAttachment>,
    pub(crate) repeated_instance_basis: UiRepeatedInstanceBasis,
    pub(crate) topology_seed: UiGraphTopologySeed,
    pub(crate) participation_seed: UiGraphParticipationSeed,
    pub(crate) attachment_posture: UiGraphAttachmentPosture,
    pub(crate) mount_eligibility_seed: UiGraphMountEligibilitySeed,
    pub(crate) core_index_contribution_seed: UiGraphCoreIndexContributionSeed,
}

impl UiGraphNodeInstantiationEntry {
    pub(crate) fn new(input: UiGraphNodeInstantiationInput) -> Self {
        let UiGraphNodeInstantiationInput {
            declaration_identity,
            authored_provenance_digest,
            measurement_constraint_modifier,
            measurement_basis_source,
            aspect_contract,
            component_reference,
            appearance_role_attachment,
            repeated_instance_basis,
            topology_seed,
            participation_seed,
            attachment_posture,
            mount_eligibility_seed,
            core_index_contribution_seed,
        } = input;
        Self {
            declaration_identity,
            authored_provenance_digest,
            measurement_constraint_modifier,
            measurement_basis_source,
            aspect_contract,
            component_reference,
            appearance_role_attachment,
            repeated_instance_basis,
            topology_seed,
            participation_seed,
            attachment_posture,
            mount_eligibility_seed,
            core_index_contribution_seed,
        }
    }

    pub fn declaration_identity(&self) -> &UiDeclarationIdentity {
        &self.declaration_identity
    }

    pub fn authored_provenance_digest(&self) -> u64 {
        self.authored_provenance_digest
    }

    pub fn aspect_contract(&self) -> &UiAspectContract {
        &self.aspect_contract
    }

    pub(crate) const fn appearance_role_attachment(
        &self,
    ) -> Option<&crate::declaration::UiAppearanceRoleAttachment> {
        self.appearance_role_attachment.as_ref()
    }

    pub(crate) const fn component_reference(&self) -> Option<&crate::capability::ComponentId> {
        self.component_reference.as_ref()
    }

    pub fn measurement_constraint_modifier(
        &self,
    ) -> Option<UiDeclaredMeasurementConstraintModifier> {
        self.measurement_constraint_modifier
    }

    pub fn measurement_basis_source(&self) -> Option<UiDeclaredMeasurementBasisSource> {
        self.measurement_basis_source
    }

    pub fn repeated_instance_basis(&self) -> &UiRepeatedInstanceBasis {
        &self.repeated_instance_basis
    }

    pub fn topology_seed(&self) -> &UiGraphTopologySeed {
        &self.topology_seed
    }

    pub fn participation_seed(&self) -> &UiGraphParticipationSeed {
        &self.participation_seed
    }

    pub fn attachment_posture(&self) -> UiGraphAttachmentPosture {
        self.attachment_posture
    }

    pub fn mount_eligibility_seed(&self) -> UiGraphMountEligibilitySeed {
        self.mount_eligibility_seed
    }

    pub fn core_index_contribution_seed(&self) -> UiGraphCoreIndexContributionSeed {
        self.core_index_contribution_seed
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiGraphInstantiationLocalDenial {
    declaration_identity: UiDeclarationIdentity,
    kind: UiGraphInstantiationLocalDenialKind,
}

impl UiGraphInstantiationLocalDenial {
    pub(crate) fn repeated_instance_basis(
        declaration_identity: UiDeclarationIdentity,
        denial: UiRepeatedInstanceBasisDenial,
    ) -> Self {
        Self {
            declaration_identity,
            kind: UiGraphInstantiationLocalDenialKind::RepeatedInstanceBasis(denial),
        }
    }

    pub(crate) fn topology(
        declaration_identity: UiDeclarationIdentity,
        denial: UiGraphTopologyLocalDenial,
    ) -> Self {
        Self {
            declaration_identity,
            kind: UiGraphInstantiationLocalDenialKind::Topology(denial),
        }
    }

    pub fn declaration_identity(&self) -> &UiDeclarationIdentity {
        &self.declaration_identity
    }

    pub fn kind(&self) -> &UiGraphInstantiationLocalDenialKind {
        &self.kind
    }

    pub fn repeated_instance_basis_denial(&self) -> Option<&UiRepeatedInstanceBasisDenial> {
        match &self.kind {
            UiGraphInstantiationLocalDenialKind::RepeatedInstanceBasis(denial) => Some(denial),
            UiGraphInstantiationLocalDenialKind::Topology(_) => None,
        }
    }

    pub fn topology_denial(&self) -> Option<&UiGraphTopologyLocalDenial> {
        match &self.kind {
            UiGraphInstantiationLocalDenialKind::RepeatedInstanceBasis(_) => None,
            UiGraphInstantiationLocalDenialKind::Topology(denial) => Some(denial),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UiGraphInstantiationLocalDenialKind {
    RepeatedInstanceBasis(UiRepeatedInstanceBasisDenial),
    Topology(UiGraphTopologyLocalDenial),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UiGraphTopologyLocalDenial {
    RootPageCardinality { observed_root_pages: usize },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiGraphTopologySeed {
    structural_digest: UiDeclarationStructuralDigest,
    role: UiDeclarationStructuralRole,
    operator_kind: UiDeclarationPlanningOperatorKind,
    containment_claim: UiGraphContainmentClaim,
    parent_resolution_claim: UiGraphParentResolutionClaim,
    slot_participation_intent: UiDeclarationSlotParticipationIntent,
    ordering_guarantee: UiDeclarationOrderingGuarantee,
    repetition_posture: UiDeclarationRepetitionPosture,
}

pub(crate) struct UiGraphTopologySeedInput {
    pub(crate) structural_digest: UiDeclarationStructuralDigest,
    pub(crate) role: UiDeclarationStructuralRole,
    pub(crate) operator_kind: UiDeclarationPlanningOperatorKind,
    pub(crate) containment_claim: UiGraphContainmentClaim,
    pub(crate) parent_resolution_claim: UiGraphParentResolutionClaim,
    pub(crate) slot_participation_intent: UiDeclarationSlotParticipationIntent,
    pub(crate) ordering_guarantee: UiDeclarationOrderingGuarantee,
    pub(crate) repetition_posture: UiDeclarationRepetitionPosture,
}

impl UiGraphTopologySeed {
    pub(crate) fn new(input: UiGraphTopologySeedInput) -> Self {
        let UiGraphTopologySeedInput {
            structural_digest,
            role,
            operator_kind,
            containment_claim,
            parent_resolution_claim,
            slot_participation_intent,
            ordering_guarantee,
            repetition_posture,
        } = input;
        Self {
            structural_digest,
            role,
            operator_kind,
            containment_claim,
            parent_resolution_claim,
            slot_participation_intent,
            ordering_guarantee,
            repetition_posture,
        }
    }

    pub fn role(&self) -> UiDeclarationStructuralRole {
        self.role
    }

    pub fn structural_digest(&self) -> UiDeclarationStructuralDigest {
        self.structural_digest
    }

    pub fn operator_kind(&self) -> UiDeclarationPlanningOperatorKind {
        self.operator_kind
    }

    pub fn containment_claim(&self) -> &UiGraphContainmentClaim {
        &self.containment_claim
    }

    pub fn parent_resolution_claim(&self) -> &UiGraphParentResolutionClaim {
        &self.parent_resolution_claim
    }

    pub fn slot_participation_intent(&self) -> &UiDeclarationSlotParticipationIntent {
        &self.slot_participation_intent
    }

    pub fn ordering_guarantee(&self) -> UiDeclarationOrderingGuarantee {
        self.ordering_guarantee
    }

    pub fn repetition_posture(&self) -> UiDeclarationRepetitionPosture {
        self.repetition_posture
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiGraphCoreIndexContributionSeed {
    declaration_correspondence: bool,
    node_identity_lookup: bool,
}

impl UiGraphCoreIndexContributionSeed {
    pub(crate) const fn authoritative() -> Self {
        Self {
            declaration_correspondence: true,
            node_identity_lookup: true,
        }
    }

    pub fn declaration_correspondence(self) -> bool {
        self.declaration_correspondence
    }

    pub fn node_identity_lookup(self) -> bool {
        self.node_identity_lookup
    }
}
