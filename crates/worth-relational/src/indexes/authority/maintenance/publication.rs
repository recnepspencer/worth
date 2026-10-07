use super::super::{publish_prepared_generation, IndexGenerationPublicationBasis};
use super::work::MaintenanceWork;
use crate::indexes::data::{DerivedIndexEntries, DerivedIndexGeneration, DerivedIndexId};
use crate::indexes::data::{
    DerivedIndexMaintenanceDenialKind as Denial, SelectedIndexGenerationAdmissionStop,
};
pub(super) struct PreparedRefresh {
    pub(super) publication: IndexGenerationPublicationBasis,
    pub(super) entries: Vec<(DerivedIndexId, DerivedIndexEntries)>,
    pub(super) reused: Vec<DerivedIndexGeneration>,
}

impl PreparedRefresh {
    pub(super) fn publish_admitted(
        self,
        runtime: &crate::runtime::RelationalRuntime,
        work: &mut MaintenanceWork,
    ) -> Result<Vec<DerivedIndexGeneration>, Denial> {
        let count = self
            .entries
            .len()
            .checked_add(self.reused.len())
            .ok_or(Denial::WorkBudgetExceeded)?;
        work.array::<DerivedIndexGeneration>(count)?;
        work.array::<crate::indexes::data::DerivedIndexGenerationId>(self.entries.len())?;
        let initialized = self
            .entries
            .len()
            .checked_add(count)
            .ok_or(Denial::WorkBudgetExceeded)?;
        work.prepare(initialized as u64, 0)?;
        let identities = runtime
            .indexes
            .reserve_generation_ids(self.entries.len())
            .ok_or(Denial::GenerationIdentityExhausted)?;
        let mut generations = Vec::with_capacity(count);
        for ((index, entries), identity) in self.entries.into_iter().zip(identities) {
            let width = self.publication.branch_id.0.len();
            work.prepare(
                (width as u64)
                    .checked_mul(2)
                    .ok_or(Denial::WorkBudgetExceeded)?,
                (width as u64)
                    .checked_mul(2)
                    .ok_or(Denial::WorkBudgetExceeded)?,
            )?;
            generations.push(super::super::prepared_generation(
                &self.publication,
                identity,
                index,
                entries,
            ));
        }
        let comparisons = count
            .checked_mul((usize::BITS - count.leading_zeros()) as usize + 1)
            .and_then(|n| n.checked_mul(4))
            .ok_or(Denial::WorkBudgetExceeded)?;
        work.prepare(comparisons as u64, 0)?;
        runtime
            .indexes
            .publish_generations_admitted(&generations, |units, bytes| work.prepare(units, bytes))
            .map_err(|stop| match stop {
                SelectedIndexGenerationAdmissionStop::Admission(stop) => stop,
                SelectedIndexGenerationAdmissionStop::AccountingOverflow => {
                    Denial::WorkBudgetExceeded
                }
            })?;
        generations.extend(self.reused);
        generations.sort_unstable_by_key(|generation| generation.index_id);
        Ok(generations)
    }
    pub(super) fn publish(
        self,
        runtime: &crate::runtime::RelationalRuntime,
    ) -> Vec<DerivedIndexGeneration> {
        let mut generations = self.reused;
        generations.extend(self.entries.into_iter().map(|(index_id, entries)| {
            publish_prepared_generation(runtime, &self.publication, index_id, entries)
        }));
        generations.sort_by_key(|generation| generation.index_id);
        generations
    }
}
