use super::scroll_binding_key_index::BindingKeyIndex;
use super::scroll_catalog_evidence::{bump, UiScrollCatalogIdentity};

type ActivationRow = crate::runtime::allocation_receipt::UiCommittedAllocationCatalogActivationRow;

impl super::UiScrollInvalidationBindingIndex {
    pub(super) fn apply_catalog_delta(
        &mut self,
        removed: &[ActivationRow],
        changed: &[ActivationRow],
        changed_identity_digest: u64,
        graph: &crate::graph::UiGraphReplanAuthority,
        predecessor_identity_digest: u64,
        successor_identity_digest: u64,
    ) -> Result<(), super::UiScrollOwnerCatalogDenialReport> {
        let mut counters = super::UiScrollBindingCatalogCounters::default();
        let mut activation_keys = Vec::new();
        let result = (|| {
            for row in removed {
                self.remove_catalog_row(row, &mut counters)?;
            }
            for row in changed {
                self.insert_catalog_row(row, graph, &mut counters, &mut activation_keys)?;
            }
            let identity = UiScrollCatalogIdentity::new(
                changed_identity_digest,
                predecessor_identity_digest,
                successor_identity_digest,
                activation_keys.clone().into_boxed_slice(),
            );
            self.catalog_receipt = Some(super::UiScrollOwnerCatalogReceipt::seal(
                counters,
                self.projection_contracts.len(),
                identity,
            )?);
            Ok(())
        })();
        result.map_err(|reason| {
            let _ = bump(&mut counters.diagnostic_probes);
            super::UiScrollOwnerCatalogDenialReport::new(
                reason,
                counters,
                UiScrollCatalogIdentity::new(
                    changed_identity_digest,
                    predecessor_identity_digest,
                    successor_identity_digest,
                    activation_keys.into_boxed_slice(),
                ),
            )
        })
    }

    fn remove_catalog_row(
        &mut self,
        row: &ActivationRow,
        counters: &mut super::UiScrollBindingCatalogCounters,
    ) -> Result<(), super::UiScrollInvalidationBindingDenial> {
        bump(&mut counters.extent_rows_visited)?;
        for source in row.scroll_sources() {
            bump(&mut counters.source_visits)?;
            let (contract, host) = match source {
                crate::runtime::allocation_receipt::UiCommittedScrollActivationSource::Host {
                    witness,
                    contract,
                } => (contract, Some(*witness)),
                crate::runtime::allocation_receipt::UiCommittedScrollActivationSource::Query {
                    contract,
                } => (contract, None),
            };
            let key = crate::runtime::UiScrollReceiptActivationKey::from_receipt_and_source(
                row.receipt(),
                contract.source().clone(),
            );
            if let Some(witness) = host {
                remove_binding(&mut self.host, &witness, &key)?;
            } else {
                let crate::runtime::UiAdmittedScrollExtentSource::QueryContent(query) =
                    contract.source()
                else {
                    return Err(super::UiScrollInvalidationBindingDenial::ContradictorySource);
                };
                remove_binding(&mut self.query, query.source_key(), &key)?;
                remove_binding(
                    &mut self.query_by_binding,
                    &query.source_key().binding_key(),
                    &key,
                )?;
            }
            if !self
                .projection_contracts
                .remove(&contract.projection_owner_identity())
            {
                return Err(super::UiScrollInvalidationBindingDenial::MissingSourceBinding);
            }
            bump(&mut counters.index_writes)?;
        }
        Ok(())
    }

    fn insert_catalog_row(
        &mut self,
        row: &ActivationRow,
        graph: &crate::graph::UiGraphReplanAuthority,
        counters: &mut super::UiScrollBindingCatalogCounters,
        activation_keys: &mut Vec<crate::runtime::UiScrollReceiptActivationKey>,
    ) -> Result<(), super::UiScrollInvalidationBindingDenial> {
        bump(&mut counters.extent_rows_visited)?;
        bump(&mut counters.context_reads)?;
        bump(&mut counters.receipt_validations)?;
        bump(&mut counters.structural_comparisons)?;
        if row.receipt_identity() != row.receipt().identity()
            || row.receipt_generation() != row.receipt().generation()
            || row.measurement_basis_identity_digest() != row.measurement_basis().identity_digest()
        {
            return Err(super::UiScrollInvalidationBindingDenial::ReceiptContextMismatch);
        }
        for source in row.scroll_sources() {
            bump(&mut counters.source_visits)?;
            let (contract, node, cause, host) = match source {
                crate::runtime::allocation_receipt::UiCommittedScrollActivationSource::Host {
                    witness,
                    contract,
                } => (
                    contract.clone(),
                    row.receipt_identity().graph_node_identity(),
                    crate::evidence::UiScrollOwnedExtentCause::HostContainerViewport,
                    Some(*witness),
                ),
                crate::runtime::allocation_receipt::UiCommittedScrollActivationSource::Query {
                    contract,
                } => {
                    let crate::runtime::UiAdmittedScrollExtentSource::QueryContent(query) =
                        contract.source()
                    else {
                        return Err(super::UiScrollInvalidationBindingDenial::ContradictorySource);
                    };
                    (
                        contract.clone(),
                        query.target(),
                        crate::evidence::UiScrollOwnedExtentCause::QueryContentExtent,
                        None,
                    )
                }
            };
            bump(&mut counters.target_probes)?;
            let target = graph
                .target_set_for_neighborhood(node, contract.neighborhood_identity())
                .ok_or(super::UiScrollInvalidationBindingDenial::MissingGraphTarget)?;
            bump(&mut counters.graph_target_validations)?;
            bump(&mut counters.targets_emitted)?;
            let binding = super::UiAdmittedScrollInvalidationBinding::seal(
                contract.clone(),
                target,
                cause,
                1,
                Some(row.receipt()),
            )?;
            let key = binding
                .receipt_key()
                .ok_or(super::UiScrollInvalidationBindingDenial::MissingReceiptBinding)?
                .clone();
            activation_keys.push(key.clone());
            bump(&mut counters.bindings_sealed)?;
            if let Some(witness) = host {
                insert_binding(&mut self.host, witness, key.clone(), binding)?;
            } else {
                let crate::runtime::UiAdmittedScrollExtentSource::QueryContent(query) =
                    contract.source()
                else {
                    return Err(super::UiScrollInvalidationBindingDenial::ContradictorySource);
                };
                insert_binding(
                    &mut self.query,
                    query.source_key().clone(),
                    key.clone(),
                    binding.clone(),
                )?;
                insert_binding(
                    &mut self.query_by_binding,
                    query.source_key().binding_key(),
                    key.clone(),
                    binding,
                )?;
            }
            bump(&mut counters.index_writes)?;
            let owner = contract.projection_owner_identity();
            let projection = (contract, node, key);
            bump(&mut counters.owner_validations)?;
            if self
                .projection_contracts
                .get(&owner)
                .is_some_and(|prior| prior != &projection)
            {
                return Err(super::UiScrollInvalidationBindingDenial::ConflictingOwnerCapability);
            }
            self.projection_contracts.insert(owner, projection);
            bump(&mut counters.projection_writes)?;
        }
        Ok(())
    }
}

pub(super) fn insert_binding<K: Ord + Clone>(
    index: &mut crate::runtime::persistent_index::UiPersistentOrdMap<K, BindingKeyIndex>,
    owner: K,
    key: crate::runtime::UiScrollReceiptActivationKey,
    binding: super::UiAdmittedScrollInvalidationBinding,
) -> Result<(), super::UiScrollInvalidationBindingDenial> {
    let mut bucket = index.get(&owner).cloned().unwrap_or_default();
    if let Some(prior) = bucket.insert(key.identity_digest(), binding.clone())
        && prior != binding
    {
        return Err(super::UiScrollInvalidationBindingDenial::ConflictingContractTarget);
    }
    index.insert(owner, bucket);
    Ok(())
}

pub(super) fn remove_binding<K: Ord + Clone>(
    index: &mut crate::runtime::persistent_index::UiPersistentOrdMap<K, BindingKeyIndex>,
    owner: &K,
    key: &crate::runtime::UiScrollReceiptActivationKey,
) -> Result<(), super::UiScrollInvalidationBindingDenial> {
    let mut bucket = index
        .get(owner)
        .cloned()
        .ok_or(super::UiScrollInvalidationBindingDenial::MissingSourceBinding)?;
    bucket
        .remove(key)
        .ok_or(super::UiScrollInvalidationBindingDenial::MissingSourceBinding)?;
    if bucket.is_empty() {
        if !index.remove(owner) {
            return Err(super::UiScrollInvalidationBindingDenial::MissingSourceBinding);
        }
    } else {
        index.insert(owner.clone(), bucket);
    }
    Ok(())
}
