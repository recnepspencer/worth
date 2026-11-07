//! Acceptance transfers candidate closure evidence together with its cold meanings.
use super::*;
use crate::storage::residency::RelationalRegionSymbolAdmissionIdentity;

#[derive(Debug)]
pub(super) struct PreparedSymbolAdmissionTransfer {
    cold: Option<Arc<super::super::symbol_residency::RuntimeSymbolResidency>>,
    additions: SymbolTableSnapshot,
    regions: Vec<RelationalRegionSymbolAdmissionIdentity>,
}
impl RuntimeSymbolTable {
    pub(super) fn prepare_admission_transfer(
        &self,
        frozen: &Self,
        pending: &StringInterner,
    ) -> Result<PreparedSymbolAdmissionTransfer, &'static str> {
        let cold = self
            .cold
            .read()
            .unwrap_or_else(|error| error.into_inner())
            .clone();
        let before = frozen
            .cold
            .read()
            .unwrap_or_else(|error| error.into_inner())
            .clone();
        match (&cold, &before) {
            (None, None) => {}
            (Some(current), Some(previous)) if current.shares_directory(previous) => {}
            _ => return Err("candidate dictionary replaced its admission authority"),
        }
        let current = self.admissions_snapshot();
        let previous = frozen.admissions_snapshot();
        let regions = previous
            .diff(&current)
            .filter_map(|entry| match entry {
                im::ordmap::DiffItem::Add(id, _) => Some(id.clone()),
                _ => None,
            })
            .collect();
        // Every carried mark was frozen with these meanings. A supplied binding
        // may add meanings; it must never erase the frozen dictionary under them.
        let base = frozen.interner_snapshot();
        let additions = self.with_read(|current| current.checked_additions_since(&base))?;
        let mut cold_additions = Vec::new();
        for (id, text) in additions.entries {
            match pending.retained_text(id) {
                Some(expected) if expected == &text => {}
                Some(_) => return Err("candidate binding changed a reserved meaning"),
                None => cold_additions.push((id, text)),
            }
        }
        Ok(PreparedSymbolAdmissionTransfer {
            cold,
            additions: SymbolTableSnapshot {
                entries: cold_additions,
            },
            regions,
        })
    }
}
impl PreparedSymbolAdmissionTransfer {
    pub(super) fn verify_required(
        &self,
        required: impl IntoIterator<Item = Symbol>,
        binding: &RuntimeSymbolTable,
        owner: &RuntimeSymbolTable,
        meanings: &SymbolTableSnapshot,
    ) -> Result<(), &'static str> {
        let published: std::collections::BTreeMap<_, _> = meanings
            .entries
            .iter()
            .chain(&self.additions.entries)
            .map(|(id, text)| (*id, text.as_str()))
            .collect();
        for id in required {
            let expected = binding
                .with_read(|table| table.resolve(id).map(str::to_owned))
                .ok_or("required symbol has no candidate meaning")?;
            if owner
                .with_read(|table| table.resolve(id).map(str::to_owned))
                .as_deref()
                != Some(expected.as_str())
                && published.get(&id).copied() != Some(expected.as_str())
            {
                return Err("required symbol is absent from accepted publication");
            }
        }
        Ok(())
    }
    pub(super) fn publish(self, owner: &RuntimeSymbolTable) -> Result<(), &'static str> {
        let target = owner
            .cold
            .read()
            .unwrap_or_else(|error| error.into_inner())
            .clone();
        match (&target, &self.cold) {
            (None, None) => {}
            (Some(target), Some(source)) if target.shares_directory(source) => {}
            _ => return Err("canonical dictionary changed before admission transfer"),
        }
        owner.import_canonical_snapshot(&self.additions)?;
        owner.publish_admissions(self.regions);
        Ok(())
    }
}

impl RuntimeSymbolTable {
    pub(in crate::runtime::state::subsystems::services) fn admissions_snapshot(
        &self,
    ) -> im::OrdMap<crate::storage::residency::RelationalRegionSymbolAdmissionIdentity, ()> {
        self.read_state(|state| state.admissions.clone())
    }
    pub(in crate::runtime::state::subsystems::services) fn publish_admissions(
        &self,
        identities: Vec<crate::storage::residency::RelationalRegionSymbolAdmissionIdentity>,
    ) {
        self.update(|state| {
            for id in identities {
                state.admissions.insert(id, ());
            }
        });
    }
    pub(crate) fn register_symbol_publication(
        &self,
        publication: &crate::storage::residency::SymbolPublicationKey,
    ) {
        self.publish_admissions(vec![publication.identity()]);
    }
    #[cfg(test)]
    pub(crate) fn symbol_publication_admitted(
        &self,
        publication: &crate::storage::residency::SymbolPublicationKey,
    ) -> bool {
        self.read_state(|state| state.admissions.contains_key(&publication.identity()))
    }
    pub(crate) fn admit_region_from_publication(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        region: crate::storage::residency::RelationalRegionSymbolAdmissionIdentity,
        publication: &crate::storage::residency::SymbolPublicationKey,
    ) -> Result<bool, crate::storage::residency::RelationalResidencyDemandFailure> {
        crate::execution::check_request(execution)
            .map_err(crate::storage::residency::RelationalResidencyDemandFailure::Request)?;
        Ok(self.update(|state| {
            if !state.admissions.contains_key(&publication.identity()) {
                return false;
            }
            if let Some(head) = region.covered_head() {
                state.admissions.insert(head, ());
            }
            if let Some(owned) = region.covered_owned() {
                state.admissions.insert(owned, ());
            }
            state.admissions.insert(region, ());
            true
        }))
    }
}
