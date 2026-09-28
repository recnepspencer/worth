use worth_foundational::facade::{AspectKey, AspectValue};
use worth_runtime_bridge::facade::BridgeGroupedTruthViewArtifact;
use worth_runtime_bridge::facade::RelationalGroupedProjectionArtifact;

use super::super::consumed::{
    ConsumedMembershipFact, ConsumedProjectionContractProvenance, ConsumedProjectionFactInventory,
    ConsumedProjectionFactSet, ConsumedProjectionSourceTruth, ConsumedRelationEndpointFact,
    ConsumedViewLocalIdentityFact, ProjectionFactExtractionCounters,
};
use super::super::contracts::MaterializedProjectionContract;
use super::super::facts::ProjectionFactKind;
use super::super::source::ProjectionSourceFamily;
use crate::projection_consumption::ProjectionFactExtractionError;

pub(super) fn extract_relational_grouped_facts(
    contract: &MaterializedProjectionContract,
    projection: &RelationalGroupedProjectionArtifact,
) -> Result<ConsumedProjectionFactSet, ProjectionFactExtractionError> {
    extract_grouped_facts(
        contract,
        ProjectionSourceFamily::RelationalGroupedProjection,
        projection.digest().as_str(),
        projection.contract().grouping_aspect(),
        projection.members().iter().map(|member| {
            ProjectionGroupedMember::new(
                member.row_identity().as_str(),
                member.identity_value().clone(),
                member.grouping_value().clone(),
            )
        }),
    )
}

pub(super) fn extract_bridge_grouped_facts(
    contract: &MaterializedProjectionContract,
    grouped_truth_view: &BridgeGroupedTruthViewArtifact,
) -> Result<ConsumedProjectionFactSet, ProjectionFactExtractionError> {
    extract_grouped_facts(
        contract,
        ProjectionSourceFamily::BridgeGroupedTruthView,
        grouped_truth_view.digest().as_str(),
        grouped_truth_view.contract().native_grouping_aspect_key(),
        grouped_truth_view.members().iter().map(|member| {
            ProjectionGroupedMember::new(
                member.row_identity().as_str(),
                member.identity_value().clone(),
                member.lane().value().clone(),
            )
        }),
    )
}

struct ProjectionGroupedMember {
    row_identity: String,
    member_identity: AspectValue,
    grouping_value: AspectValue,
}

impl ProjectionGroupedMember {
    fn new(row_identity: &str, member_identity: AspectValue, grouping_value: AspectValue) -> Self {
        Self {
            row_identity: row_identity.to_string(),
            member_identity,
            grouping_value,
        }
    }

    fn row_identity(&self) -> &str {
        &self.row_identity
    }

    fn member_identity(&self) -> &AspectValue {
        &self.member_identity
    }

    fn grouping_value(&self) -> &AspectValue {
        &self.grouping_value
    }
}

fn extract_grouped_facts<Members>(
    contract: &MaterializedProjectionContract,
    expected_family: ProjectionSourceFamily,
    source_identity: &str,
    grouping_aspect: &AspectKey,
    members: Members,
) -> Result<ConsumedProjectionFactSet, ProjectionFactExtractionError>
where
    Members: Iterator<Item = ProjectionGroupedMember>,
{
    super::ensure_contract_family(contract, expected_family)?;
    super::ensure_source_identity(contract.source_identity(), source_identity)?;

    let materialized_members = members.collect::<Vec<_>>();
    let extracts_view_local_identity = contract
        .fact_families()
        .iter()
        .any(|fact| fact.kind() == ProjectionFactKind::ViewLocalIdentity);
    let extracts_membership = contract
        .fact_families()
        .iter()
        .any(|fact| fact.kind() == ProjectionFactKind::Membership);
    let extracts_relation_endpoint = contract
        .fact_families()
        .iter()
        .any(|fact| fact.kind() == ProjectionFactKind::RelationEndpoint);
    let mut view_local_identities = Vec::new();
    let mut memberships = Vec::new();
    let mut relation_endpoints = Vec::new();

    for member in &materialized_members {
        for fact_family in contract.fact_families() {
            match fact_family.kind() {
                ProjectionFactKind::ViewLocalIdentity => {
                    view_local_identities.push(ConsumedViewLocalIdentityFact::new(
                        member.row_identity(),
                        member.row_identity(),
                    ));
                }
                ProjectionFactKind::Membership => {
                    memberships.push(ConsumedMembershipFact::new(
                        member.row_identity(),
                        member.member_identity().clone(),
                        grouping_aspect.clone(),
                        member.grouping_value().clone(),
                    ));
                }
                ProjectionFactKind::RelationEndpoint => {
                    relation_endpoints.push(ConsumedRelationEndpointFact::grouped(
                        member.row_identity(),
                        member.member_identity().clone(),
                        grouping_aspect.clone(),
                        member.grouping_value().clone(),
                    ));
                }
                ProjectionFactKind::EntityIdentity
                | ProjectionFactKind::TargetIdentity
                | ProjectionFactKind::SourceReference
                | ProjectionFactKind::EffectContinuity
                | ProjectionFactKind::DisplayField
                | ProjectionFactKind::DerivedField => {}
            }
        }
    }

    let row_count = materialized_members.len();
    let row_identity_surface_count =
        usize::from(extracts_view_local_identity || extracts_relation_endpoint);
    let member_identity_surface_count =
        usize::from(extracts_membership || extracts_relation_endpoint);
    let grouping_value_surface_count =
        usize::from(extracts_membership || extracts_relation_endpoint);
    let row_width_per_row =
        row_identity_surface_count + member_identity_surface_count + grouping_value_surface_count;
    let source_row_width_consumed = row_count * row_width_per_row;
    let extracted_fact_count =
        view_local_identities.len() + memberships.len() + relation_endpoints.len();

    Ok(ConsumedProjectionFactSet::new(
        ConsumedProjectionContractProvenance::from_contract(contract),
        ConsumedProjectionSourceTruth::from_contract(
            contract,
            crate::projection_consumption::ConsumedNativeLayoutProof::from_contract(
                contract, row_count,
            ),
        ),
        ProjectionFactExtractionCounters::new(
            contract.fact_families().len(),
            contract.fact_families().len(),
            extracted_fact_count,
            source_row_width_consumed,
            0,
        ),
        ConsumedProjectionFactInventory {
            entity_identities: Vec::new(),
            view_local_identities,
            memberships,
            display_fields: Vec::new(),
            derived_fields: Vec::new(),
            target_identities: Vec::new(),
            source_references: Vec::new(),
            effect_continuity_facts: Vec::new(),
            relation_endpoints,
        },
    ))
}
