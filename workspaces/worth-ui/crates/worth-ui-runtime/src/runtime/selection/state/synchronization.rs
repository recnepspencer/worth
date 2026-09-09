use crate::runtime::persistent_index::UiPersistentOrdMap;
use std::collections::BTreeMap;

impl super::UiSelectionRuntimeState {
    pub(crate) fn synchronize(
        &mut self,
        registration: super::super::UiSelectionRegistration,
    ) -> Result<
        super::super::UiSelectionReconciliationReceipt,
        super::super::UiSelectionRequestDenial,
    > {
        let owner = registration.owner();
        if registration.catalog_revision() != 0
            && self.catalog_is_current(
                owner,
                registration.incarnation(),
                registration.catalog_revision(),
            )
        {
            let selected = self
                .owners
                .get(&owner)
                .map_or(0, |record| record.selected.len());
            let positions = self
                .owners
                .get(&owner)
                .map(|record| record.positions())
                .unwrap_or_default();
            return Ok(super::super::UiSelectionReconciliationReceipt::new(
                super::super::UiSelectionDelta::new(
                    Vec::new(),
                    Vec::new(),
                    selected,
                    0,
                    self.revision,
                    super::super::UiSelectionPositionChanges::new(positions, positions),
                ),
                false,
                0,
            ));
        }
        let revision = self
            .revision
            .checked_add(1)
            .ok_or(super::super::UiSelectionRequestDenial::RevisionExhausted)?;
        let catalog_keys_reconciled = self
            .catalog_keys_reconciled
            .checked_add(u64::try_from(registration.catalog().len()).unwrap_or(u64::MAX))
            .ok_or(super::super::UiSelectionRequestDenial::CounterOverflow)?;
        let prior_incarnation = self.owners.get(&owner).map(|record| record.incarnation);
        let previous_positions = self
            .owners
            .get(&owner)
            .map(|record| record.positions())
            .unwrap_or_default();
        let mut record = self
            .owners
            .get(&owner)
            .cloned()
            .unwrap_or_else(|| super::record::empty_record(&registration));
        let (order_changed, removed, missing_count, selected_count) = {
            if record.incarnation != registration.incarnation() {
                record = super::record::empty_record(&registration);
            }
            let order_changed = record.catalog.as_ref() != registration.catalog();
            let available = registration.catalog_positions();
            let missing = record
                .selected
                .iter()
                .filter(|key| !available.contains_key(key))
                .copied()
                .collect::<Vec<_>>();
            let complete =
                registration.catalog_posture() == super::super::UiSelectionCatalogPosture::Complete;
            let remove_missing = complete || !self.policy.preserves_stable_keys();
            let removed = if remove_missing {
                for key in &missing {
                    record.selected.remove(key);
                }
                if record
                    .anchor
                    .is_some_and(|key| !available.contains_key(&key))
                {
                    record.anchor = None;
                }
                if record
                    .cursor
                    .is_some_and(|key| !available.contains_key(&key))
                {
                    record.cursor = None;
                }
                missing.clone()
            } else {
                Vec::new()
            };
            record.policy = registration.policy();
            record.catalog = registration.catalog().to_vec().into();
            record.catalog_positions = std::sync::Arc::clone(registration.catalog_positions());
            record.catalog_posture = registration.catalog_posture();
            record.catalog_revision = registration.catalog_revision();
            record.catalog_available = true;
            (
                order_changed,
                removed,
                if remove_missing { 0 } else { missing.len() },
                record.selected.len(),
            )
        };
        let positions =
            super::super::UiSelectionPositionChanges::new(previous_positions, record.positions());
        record.revision = revision;
        self.owners.insert(owner, record);
        if prior_incarnation != Some(registration.incarnation()) {
            if let Some(prior) = prior_incarnation {
                self.unindex_owner(owner, prior);
            }
            self.index_owner(owner, registration.incarnation());
        }
        self.revision = revision;
        self.catalog_keys_reconciled = catalog_keys_reconciled;
        let receipt = super::super::UiSelectionReconciliationReceipt::new(
            super::super::UiSelectionDelta::new(
                Vec::new(),
                removed,
                selected_count,
                u32::try_from(registration.catalog().len()).unwrap_or(u32::MAX),
                revision,
                positions,
            ),
            order_changed,
            missing_count,
        );
        self.record_drop(
            owner,
            super::super::UiSelectionDropInspectionReason::CatalogReconciliation,
            receipt.delta(),
        );
        Ok(receipt)
    }

    pub(crate) fn synchronize_and_apply(
        &mut self,
        registration: super::super::UiSelectionRegistration,
        request: super::super::UiSelectionRequest,
    ) -> Result<
        (
            super::super::UiSelectionReconciliationReceipt,
            super::super::UiSelectionDelta,
        ),
        super::super::UiSelectionRequestDenial,
    > {
        let owner = registration.owner();
        let incarnation = registration.incarnation();
        let mut staged = Self {
            persistence: self.persistence,
            policy: self.policy,
            owners: self
                .owners
                .get(&owner)
                .cloned()
                .map(|record| {
                    let mut owners = UiPersistentOrdMap::new();
                    owners.insert(owner, record);
                    owners
                })
                .unwrap_or_default(),
            revision: self.revision,
            requests: self.requests,
            candidates_visited: self.candidates_visited,
            catalog_keys_reconciled: self.catalog_keys_reconciled,
            mounted_owners: UiPersistentOrdMap::new(),
            family_owners: BTreeMap::new(),
            last_drop: self.last_drop,
        };
        if let Some(prior) = staged.owners.get(&owner).map(|record| record.incarnation) {
            staged.index_owner(owner, prior);
        }
        let reconciliation = staged.synchronize(registration)?;
        let delta = staged.apply(owner, incarnation, request)?;
        let record = staged
            .owners
            .get(&owner)
            .cloned()
            .expect("successful staged selection retains its exact owner");
        let prior_incarnation = self.owners.get(&owner).map(|record| record.incarnation);
        self.owners.insert(owner, record);
        if prior_incarnation != Some(incarnation) {
            if let Some(prior) = prior_incarnation {
                self.unindex_owner(owner, prior);
            }
            self.index_owner(owner, incarnation);
        }
        self.revision = staged.revision;
        self.requests = staged.requests;
        self.candidates_visited = staged.candidates_visited;
        self.catalog_keys_reconciled = staged.catalog_keys_reconciled;
        self.last_drop = staged.last_drop;
        Ok((reconciliation, delta))
    }
}
