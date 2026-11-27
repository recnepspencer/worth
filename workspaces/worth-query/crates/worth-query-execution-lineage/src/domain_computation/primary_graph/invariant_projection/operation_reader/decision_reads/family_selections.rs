//! Issued family selection VALUES travel with this projection; keys carry no proof (public #429).
use super::*;
use crate::domain_computation::primary_graph::application_attempt::FamilySelectionRead;
use crate::domain_computation::primary_graph::output_lineage::family_selection::ObservedFamilySelection;
use std::sync::Arc;
use worth_relational::facade::mvcc::CompanionPreflightStop as Stop;

type Value = (FamilySelectionRead, Arc<ObservedFamilySelection>);

pub struct IssuedFamilySelectionValues {
    pub(super) values: Vec<Value>,
}

impl IssuedFamilySelectionValues {
    pub fn get(&self, key: &FamilySelectionRead) -> Option<&Arc<ObservedFamilySelection>> {
        self.values.iter().find(|(issued, _)| issued == key).map(|(_, value)| value)
    }
}

impl DecisionReads {
    /// The first selection a read makes for its key is the one it retains:
    /// later reads under the same key reuse the reader's certified family.
    pub fn record_family_selection(
        &mut self,
        key: FamilySelectionRead,
        value: Arc<ObservedFamilySelection>,
    ) -> Result<(), Stop> {
        let prior = self.family_selections.as_deref();
        if prior.is_none_or(|prior| prior.get(&key).is_none()) {
            let count = prior.map_or(0, |prior| prior.values.len());
            let len = count.checked_add(1).ok_or(Stop::PreparationMemoryCounterOverflow)?;
            let mut values = Vec::with_capacity(len);
            if let Some(prior) = prior {
                values.extend(prior.values.iter().cloned());
            }
            values.push((key.clone(), value));
            self.family_selections = Some(Arc::new(IssuedFamilySelectionValues { values }));
        }
        self.record(WorthQueryApplicationFactKey::FamilySelection(key), DecisionReadOutcome::Observed);
        Ok(())
    }
}
