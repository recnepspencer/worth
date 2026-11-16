//! Seal revisions at the exact image that supplied the compiled acquisition.
//! This visits recorded acquisition coordinates; it never rediscovers Source rows.
use super::*;
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationObservedFact as Fact;
use worth_relational::facade::{
    runtime::{
        AdjacencyStructuralRevisionDenial, RelationalAdjacencyDirection,
        RelationalAppliedCandidateReadIdentity, RelationalBorrowedRecordReadDenial,
        RelationalPinnedRead,
    },
    schema::DerivedRelationDenial,
    transactions::RelationalExecutionDenialCause as Cause,
};

pub(in crate::domain_computation::primary_graph::application_query) struct AppliedSourceAcquisitionSupport
{
    candidate: RelationalAppliedCandidateReadIdentity,
    facts: Vec<Fact>,
    commitment: [u8; 32],
    _allocation: Arc<SourceFactsAllocation>,
}
impl AppliedSourceAcquisitionSupport {
    pub(in crate::domain_computation::primary_graph::application_query) fn commitment(
        &self,
    ) -> [u8; 32] {
        self.commitment
    }

    pub(in crate::domain_computation::primary_graph::application_query) fn candidate(
        &self,
    ) -> &RelationalAppliedCandidateReadIdentity {
        &self.candidate
    }
    pub(in crate::domain_computation::primary_graph::application_query) fn facts(&self) -> &[Fact] {
        &self.facts
    }
}
impl SourceProjectionSupport {
    pub(in crate::domain_computation::primary_graph::application_query::managed_index) fn seal(
        self,
        execution: &ExecutionRequest<'_, '_>,
        pin: &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
    ) -> Result<AppliedSourceAcquisitionSupport, MaterialDenial> {
        let projection = pin.project_candidate();
        self._allocation
            .claim_slice::<Fact>(execution, self.dependencies.len())?;
        let mut facts = Vec::new();
        facts
            .try_reserve_exact(self.dependencies.len())
            .map_err(|_| Cause::AllocationFailed)?;
        for dependency in self.dependencies {
            execution.consult().map_err(Cause::from)?;
            let fact = match dependency {
                ViewDependency::Entity(entity_id) => Fact::SourceEntity { entity_id },
                ViewDependency::Aspect(entity_id, aspect) => {
                    let revision = projection
                        .read_entity_aspect_version(execution, entity_id, &aspect)
                        .map_err(|cause| match cause {
                            RelationalBorrowedRecordReadDenial::Execution(cause) => {
                                MaterialDenial::Storage(cause)
                            }
                            _ => unavailable(),
                        })?
                        .ok_or_else(unavailable)?;
                    Fact::SourceAspectRevision {
                        entity_id,
                        aspect,
                        native_revision: revision,
                    }
                }
                ViewDependency::Field(entity_id, locator) => {
                    // The compiled recipe names a planned field, while the
                    // admitted Source loan reads authoritative row content.
                    // Revision support must name that same physical coordinate.
                    let locator = worth_foundational::facade::AspectFieldLocator::new(
                        worth_foundational::facade::LocatorAuthority::Authoritative,
                        locator.aspect().aspect_key().clone(),
                        locator.field_path().clone(),
                    );
                    let revision =
                        projection.entity_field_revision(execution, entity_id, &locator)?;
                    Fact::SourceFieldRevision {
                        entity_id,
                        locator,
                        native_revision: revision,
                    }
                }
                ViewDependency::Adjacency(anchor, relation_kind, direction) => {
                    let direction = match direction {
                        0 => RelationalAdjacencyDirection::Outgoing,
                        1 => RelationalAdjacencyDirection::Incoming,
                        _ => return Err(unavailable()),
                    };
                    let revision = projection.bounded_adjacency_structural_revision(
                        execution, anchor, relation_kind, direction, 1,
                    ).map_err(|cause| match cause {
                        AdjacencyStructuralRevisionDenial::Execution(cause) => MaterialDenial::Storage(cause),
                        AdjacencyStructuralRevisionDenial::WorkBudgetExceeded => MaterialDenial::Native(
                            worth_relational::facade::runtime::DerivedRelationReadDenial::Definition(
                                DerivedRelationDenial::WorkBudgetExceeded,
                            ).into()),
                        _ => unavailable(),
                    })?;
                    Fact::SourceAdjacencyRevision {
                        anchor, relation_kind, direction,
                        native_revision: revision.revision(),
                        comparison_work_limit: revision.work_units(),
                        endpoints: crate::domain_computation::primary_graph::WorthQueryApplicationSourceAdjacencyEndpoints::empty(),
                    }
                }
            };
            facts.push(fact);
        }
        let commitment = super::commitment::capture(execution, &facts)?;
        Ok(AppliedSourceAcquisitionSupport {
            commitment,
            candidate: pin.candidate_identity().clone(),
            facts,
            _allocation: self._allocation,
        })
    }
}
fn unavailable() -> MaterialDenial {
    MaterialDenial::Native(
        worth_relational::facade::runtime::ManagedIndexSourcePreparationDenial::ForeignSource,
    )
}
