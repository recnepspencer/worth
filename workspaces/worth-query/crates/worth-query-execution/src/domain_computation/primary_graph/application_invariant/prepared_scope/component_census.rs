//! Frozen complete census proofs include the genuine empty-source authority.
use super::*;
use worth_foundational::facade::AggregateDefinitionIdentity;
use worth_relational::facade::{
    runtime::{AggregateCensusReadIdentity, PreparedAggregateCensus, StructuralRelationView},
    schema::NativeMemberKey,
};

#[derive(Clone)]
pub(in crate::domain_computation::primary_graph::application_invariant) enum PreparedComponentCensus
{
    Entity(PreparedAggregateCensus),
    Projected(PreparedAggregateCensus<NativeMemberKey>),
}
impl ApplicationInvariantAdmissionAccess {
    pub(in crate::domain_computation::primary_graph::application_invariant) fn component_census(
        &self,
        key: AggregateCensusReadIdentity,
        view: &StructuralRelationView<'_>,
        definition: AggregateDefinitionIdentity,
        projected: bool,
    ) -> Result<PreparedComponentCensus, WorthQueryInvariantAccessDenial> {
        let budget = view.read_budget();
        budget.check_live().map_err(retention_denial)?;
        match self {
            Self::Planning(admission) => {
                let mut admission = admission
                    .lock()
                    .expect("application invariant admission mutex must not be poisoned");
                if let Some(prepared) = admission.censuses.get(&key) {
                    return Ok(prepared.clone());
                }
                let port = view.aggregates();
                let prepared = if projected {
                    PreparedComponentCensus::Projected(
                        port.prepare_projected_census_of_definition(definition)
                            .map_err(super::super::typed_view::component::aggregate_denial)?,
                    )
                } else {
                    PreparedComponentCensus::Entity(
                        port.prepare_census_of_definition(definition)
                            .map_err(super::super::typed_view::component::aggregate_denial)?,
                    )
                };
                budget
                    .claim_retained(super::retention_extent::census_entry_bytes())
                    .map_err(retention_denial)?;
                admission.censuses.insert(key, prepared.clone());
                Ok(prepared)
            }
            Self::Evaluation(admission) => admission
                .censuses
                .get(&key)
                .cloned()
                .ok_or_else(|| denial("derived component census")),
        }
    }
}
