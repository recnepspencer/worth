//! Exact restored-output locators over immutable checkpoint readmission authority.

use std::collections::HashMap;

use crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding;

use super::WorthQueryReadmittedAcceptedOutput;

#[derive(Default)]
pub(in crate::domain_computation::primary_graph) struct WorthQueryRecoveredOutputs {
    records: Vec<WorthQueryReadmittedAcceptedOutput>,
    by_source_partition: HashMap<(WorthQueryOperationScopeEntityBinding, [u8; 32]), Vec<usize>>,
}

impl WorthQueryRecoveredOutputs {
    pub(in crate::domain_computation::primary_graph) fn from_records(
        records: Vec<WorthQueryReadmittedAcceptedOutput>,
    ) -> Self {
        let mut by_source_partition = HashMap::new();
        for (slot, output) in records.iter().enumerate() {
            by_source_partition
                .entry((output.checkpoint.scope, output.checkpoint.source_partition))
                .or_insert_with(Vec::new)
                .push(slot);
        }
        Self {
            records,
            by_source_partition,
        }
    }

    pub(in crate::domain_computation::primary_graph) fn iter(
        &self,
    ) -> impl Iterator<Item = &WorthQueryReadmittedAcceptedOutput> {
        self.records.iter()
    }

    pub(in crate::domain_computation::primary_graph) fn matching_source_partition(
        &self,
        scope: WorthQueryOperationScopeEntityBinding,
        partition: [u8; 32],
    ) -> impl Iterator<Item = &WorthQueryReadmittedAcceptedOutput> {
        self.by_source_partition
            .get(&(scope, partition))
            .into_iter()
            .flat_map(|slots| slots.iter())
            .map(move |slot| {
                let output = &self.records[*slot];
                assert!(
                    output.checkpoint.scope == scope
                        && output.checkpoint.source_partition == partition,
                    "a restored-output locator must reference its exact source partition"
                );
                output
            })
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::WorthQueryRecoveredOutputs;
    use crate::domain_computation::primary_graph::{
        application_output_demand::{
            WorthQueryAcceptedOutputCheckpointIdentity, WorthQueryReadmittedAcceptedOutput,
        },
        WorthQueryApplicationOutputCorrespondence,
    };

    fn partition(number: u64) -> [u8; 32] {
        let mut identity = [0; 32];
        identity[..8].copy_from_slice(&number.to_le_bytes());
        identity
    }

    fn unrelated_restored_selection(population: u64) {
        let scope = crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(
            worth_relational::facade::identity::EntityId::new(
                worth_relational::facade::identity::PartitionId::main(),
                1,
                1,
            ),
        );
        let correspondence = Arc::new(WorthQueryApplicationOutputCorrespondence::default());
        let records = (0..=population)
            .map(|number| WorthQueryReadmittedAcceptedOutput {
                checkpoint: WorthQueryAcceptedOutputCheckpointIdentity {
                    producer: "producer".to_owned(),
                    source: [0x31; 32],
                    scope,
                    source_partition: partition(number),
                    producer_dependency: None,
                    idempotency_key: [0x41; 32],
                    resources: None,
                    roles: Vec::new(),
                    producer_facts: None,
                },
                correspondence: Arc::clone(&correspondence),
            })
            .collect();
        let recovered = WorthQueryRecoveredOutputs::from_records(records);
        assert_eq!(recovered.iter().count(), (population + 1) as usize);
        let selected = recovered
            .matching_source_partition(scope, partition(0))
            .collect::<Vec<_>>();
        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].checkpoint.source_partition, partition(0));
        assert!(recovered
            .matching_source_partition(scope, [0xff; 32])
            .next()
            .is_none());
    }

    #[test]
    fn restored_selection_excludes_1k_and_10k_unrelated_partitions() {
        for population in [1_000, 10_000] {
            unrelated_restored_selection(population);
        }
    }

    #[test]
    #[ignore = "100k restored partitions are a scheduled scale court; run with --ignored"]
    fn restored_selection_excludes_100k_unrelated_partitions() {
        unrelated_restored_selection(100_000);
    }
}
