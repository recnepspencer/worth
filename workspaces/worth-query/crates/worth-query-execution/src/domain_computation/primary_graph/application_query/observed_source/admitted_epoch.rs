//! Meter the actual source-epoch projection before retaining its meaning Arc.

use super::{source_identity::WorthQueryObservedSourceEpoch, WorthQueryObservedSource};
use crate::domain_computation::primary_graph::application_query::resource_lifecycle::WorthQueryApplicationBasisSelectionIdentity;

pub(in crate::domain_computation::primary_graph) enum WorthQueryObservedEpochStop<Stop> {
    AccountingOverflow,
    Admission(Stop),
}

impl<Query> WorthQueryObservedSource<Query> {
    pub(in crate::domain_computation::primary_graph) fn output_source_epoch_admitted<Stop>(
        &self,
        admit: &mut impl FnMut(u64, u64) -> Result<(), Stop>,
    ) -> Result<Option<WorthQueryObservedSourceEpoch>, WorthQueryObservedEpochStop<Stop>> {
        admit(1, 0).map_err(WorthQueryObservedEpochStop::Admission)?;
        if !matches!(
            &self.selection,
            WorthQueryApplicationBasisSelectionIdentity::Product(_)
        ) {
            return Ok(None);
        }
        // from_observation compares query/parameter digests, root and exact
        // Product occurrence, then initializes a fixed-size epoch and retains
        // its immutable source meaning. There is no nested heap copy.
        let entity = std::mem::size_of::<worth_relational::facade::identity::EntityId>();
        let occurrence =
            std::mem::size_of::<worth_runtime_world::facade::ProductBranchIncarnation>();
        let initialized = std::mem::size_of::<WorthQueryObservedSourceEpoch>();
        let entities = entity
            .checked_mul(2)
            .ok_or(WorthQueryObservedEpochStop::AccountingOverflow)?;
        let occurrences = occurrence
            .checked_mul(2)
            .ok_or(WorthQueryObservedEpochStop::AccountingOverflow)?;
        let work = initialized
            .checked_add(128)
            .and_then(|work| work.checked_add(entities))
            .and_then(|work| work.checked_add(occurrences))
            .and_then(|work| work.checked_add(12))
            .ok_or(WorthQueryObservedEpochStop::AccountingOverflow)?;
        admit(
            u64::try_from(work).map_err(|_| WorthQueryObservedEpochStop::AccountingOverflow)?,
            0,
        )
        .map_err(WorthQueryObservedEpochStop::Admission)?;
        Ok(self.output_source_epoch())
    }
}
