use crate::declaration::UiDeclarationGraphHandoff;
use crate::graph::{
    UiGraphAttachmentPosture, UiGraphContainmentClaim, UiGraphCoreIndexContributionSeed,
    UiGraphMountEligibilitySeed, UiGraphNodeInstantiationEntry, UiGraphNodeInstantiationInput,
    UiGraphParentResolutionClaim, UiGraphParticipationSeed, UiGraphTopologySeed,
    UiGraphTopologySeedInput, UiRepeatedInstanceBasis,
};

pub(super) fn construct_instantiation_entry(
    handoff: &UiDeclarationGraphHandoff,
    repeated_instance_basis: UiRepeatedInstanceBasis,
) -> UiGraphNodeInstantiationEntry {
    UiGraphNodeInstantiationEntry::new(UiGraphNodeInstantiationInput {
        declaration_identity: handoff.identity().clone(),
        authored_provenance_digest: handoff.authored_provenance_digest(),
        measurement_constraint_modifier: handoff
            .measurement_policy()
            .admitted()
            .and_then(|policy| policy.constraint_modifier()),
        measurement_basis_source: handoff
            .measurement_policy()
            .admitted()
            .and_then(|policy| policy.basis_source()),
        aspect_contract: handoff.aspect_contract().clone(),
        component_reference: handoff.component_reference().cloned(),
        appearance_role_attachment: handoff.appearance_role_attachment().cloned(),
        repeated_instance_basis,
        topology_seed: UiGraphTopologySeed::new(UiGraphTopologySeedInput {
            structural_digest: handoff.structural_digest(),
            role: handoff.role(),
            operator_kind: handoff.operator_kind(),
            containment_claim: UiGraphContainmentClaim::from_declaration_intent(
                handoff.containment_intent(),
                handoff.mosaic_sizing_contract_id().cloned(),
            ),
            parent_resolution_claim: parent_resolution_claim_for_handoff(handoff),
            slot_participation_intent: handoff.slot_participation_intent().clone(),
            ordering_guarantee: handoff.ordering_guarantee(),
            repetition_posture: handoff.repetition_posture(),
        }),
        participation_seed: UiGraphParticipationSeed::from_attachment_and_role(
            handoff.query_binding().admitted().is_some(),
            handoff.service_usage().admitted().is_some(),
            matches!(
                handoff.role(),
                crate::declaration::UiDeclarationStructuralRole::DiagnosticSurface
            ),
        ),
        attachment_posture: UiGraphAttachmentPosture::new(
            handoff.query_binding().admitted().is_some(),
            handoff.service_usage().admitted().is_some(),
        ),
        mount_eligibility_seed: UiGraphMountEligibilitySeed::reserved(),
        core_index_contribution_seed: UiGraphCoreIndexContributionSeed::authoritative(),
    })
}

fn parent_resolution_claim_for_handoff(
    handoff: &UiDeclarationGraphHandoff,
) -> UiGraphParentResolutionClaim {
    if handoff.containment_intent().is_root() {
        UiGraphParentResolutionClaim::RootPage
    } else {
        UiGraphParentResolutionClaim::ContainedByRootPage
    }
}
