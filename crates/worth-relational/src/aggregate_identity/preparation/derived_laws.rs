//! Canonical source-law admission before aggregate consequence preparation.
use super::super::derived_source;
use super::super::funding::{AggregateWork, BuildResult, PlanDenial};
use crate::derived_relation::{affected, laws, DerivedRelationHandle};
use crate::schema::data::RelationalSchemaRegistry;
use crate::validation::engine::state_view::InvariantStateView;
use std::sync::Arc;
use worth_execution::MapKernelFailure;

#[derive(Clone, Copy)]
pub(in crate::aggregate_identity) enum LawStage {
    Business,
    FinalMetadata,
}

pub(in crate::aggregate_identity) fn validate(
    registry: &RelationalSchemaRegistry,
    before: &InvariantStateView<'_>,
    after: &InvariantStateView<'_>,
    adjacency: &[crate::authority::mutation::AdjacencyDelta],
    records: &[crate::transactions::data::RecordRef],
    stage: LawStage,
    work: &mut AggregateWork<'_, '_, '_, '_>,
) -> BuildResult<()> {
    for entry in registry.derived_relations.values() {
        work.checkpoint(1)?;
        if entry.parts.laws.is_none() {
            continue;
        }
        let levels = u64::from(
            usize::BITS
                - registry
                    .aggregate_private_kinds
                    .len()
                    .max(1)
                    .leading_zeros(),
        );
        let probes = entry
            .leaves
            .len()
            .checked_mul(3)
            .and_then(|n| n.checked_add(entry.carriers.len()))
            .ok_or(MapKernelFailure::ResultCapacityExceeded)?;
        work.checkpoint(
            (probes as u64)
                .checked_mul(11)
                .and_then(|n| n.checked_mul(levels))
                .ok_or(MapKernelFailure::ResultCapacityExceeded)?,
        )?;
        let metadata =
            crate::schema::aggregate_identity::private_sources::metadata_dependent(registry, entry);
        if metadata != matches!(stage, LawStage::FinalMetadata) {
            continue;
        }
        let definition = DerivedRelationHandle {
            entry: Arc::clone(entry),
        };
        let violation = derived_source::with_views(before, after, work, |before, after| {
            let affected =
                affected::collect(registry, &definition, adjacency, records, before, after)?;
            laws::evaluate::validate(registry, &definition, &affected, before, after)
        })?;
        if let Some(violation) = violation {
            let identity = &definition.parts().identity;
            let names = identity
                .schema
                .0
                .len()
                .checked_add(identity.name.len())
                .ok_or(MapKernelFailure::ResultCapacityExceeded)?;
            work.checkpoint(names as u64)?;
            work.retain(names as u64)?;
            return work.invalid(PlanDenial::DerivedLaw {
                definition: identity.clone(),
                violation: Box::new(violation),
            });
        }
    }
    Ok(())
}
