//! Exact restored-output locators; reconstructed storage keeps its bootstrap payer.
use super::WorthQueryReadmittedAcceptedOutput;
use crate::domain_computation::{
    authorization::WorthQueryOperationScopeEntityBinding,
    primary_graph::{
        application_checkpoint::{
            CheckpointDecodeDenial, CheckpointMetadataFunding, DecodedApplicationCheckpoint,
        },
        WorthQueryPrimaryGraphInstallationDenial as Denial,
    },
};

#[derive(Default)]
pub(in crate::domain_computation::primary_graph) struct WorthQueryRecoveredOutputs {
    records: Vec<WorthQueryReadmittedAcceptedOutput>,
    by_source_partition: Vec<usize>,
    // Declared last: every retained row/string/role/wire and index drops first.
    _metadata: Option<CheckpointMetadataFunding>,
}
impl WorthQueryRecoveredOutputs {
    pub(in crate::domain_computation::primary_graph) fn from_checkpoint(
        checkpoint: DecodedApplicationCheckpoint,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        mut readmit: impl FnMut(
            &super::WorthQueryAcceptedOutputCheckpointIdentity,
        ) -> Result<
            std::sync::Arc<
                crate::domain_computation::primary_graph::WorthQueryApplicationOutputCorrespondence,
            >,
            CheckpointDecodeDenial,
        >,
    ) -> Result<Self, Denial> {
        let DecodedApplicationCheckpoint {
            accepted_outputs,
            mut metadata,
            ..
        } = checkpoint;
        let count = accepted_outputs.len();
        let old_capacity = accepted_outputs.capacity();
        let mut records = metadata
            .vec::<WorthQueryReadmittedAcceptedOutput>(count, execution)
            .map_err(CheckpointDecodeDenial::into_installation_denial)?;
        for accepted in accepted_outputs {
            metadata
                .check_live(execution)
                .map_err(CheckpointDecodeDenial::into_installation_denial)?;
            let correspondence =
                readmit(&accepted).map_err(CheckpointDecodeDenial::into_installation_denial)?;
            records.push(WorthQueryReadmittedAcceptedOutput {
                checkpoint: accepted,
                correspondence,
            });
        }
        // The consumed Vec allocation is gone; only its moved nested heaps remain.
        metadata
            .release_slots::<super::WorthQueryAcceptedOutputCheckpointIdentity>(old_capacity)
            .map_err(CheckpointDecodeDenial::into_installation_denial)?;
        Self::from_records(records, metadata, execution)
            .map_err(CheckpointDecodeDenial::into_installation_denial)
    }
    fn from_records(
        records: Vec<WorthQueryReadmittedAcceptedOutput>,
        mut metadata: CheckpointMetadataFunding,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Self, CheckpointDecodeDenial> {
        let mut by_source_partition = metadata.vec::<usize>(records.len(), execution)?;
        by_source_partition.extend(0..records.len());
        by_source_partition.sort_unstable_by_key(|slot| {
            (
                records[*slot].checkpoint.scope,
                records[*slot].checkpoint.source_partition,
            )
        });
        metadata.check_live(execution)?;
        Ok(Self {
            records,
            by_source_partition,
            _metadata: Some(metadata),
        })
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
        let key = (scope, partition);
        let start = self.by_source_partition.partition_point(|slot| {
            (
                self.records[*slot].checkpoint.scope,
                self.records[*slot].checkpoint.source_partition,
            ) < key
        });
        self.by_source_partition[start..]
            .iter()
            .map(|slot| &self.records[*slot])
            .take_while(move |output| {
                (output.checkpoint.scope, output.checkpoint.source_partition) == key
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
        let host = worth_execution::SerialRequest::from_memory(
            worth_execution::SerialMemoryBudget::new(None),
            worth_execution::CancellationToken::new(),
            None,
        );
        let execution = worth_execution::ExecutionRequest::serial(&host);
        let mut metadata = super::CheckpointMetadataFunding::new(&execution).unwrap();
        metadata
            .slots::<WorthQueryReadmittedAcceptedOutput>((population + 1) as usize, &execution)
            .unwrap();
        metadata
            .bytes("producer".len() * (population + 1) as usize, &execution)
            .unwrap();
        metadata
            .slots::<String>((population + 1) as usize, &execution)
            .unwrap();
        metadata.slots::<Vec<crate::domain_computation::primary_graph::application_attempt::WorthQueryCheckpointOutputRole>>((population + 1) as usize, &execution).unwrap();
        let records = (0..=population)
            .map(|number| WorthQueryReadmittedAcceptedOutput {
                checkpoint: WorthQueryAcceptedOutputCheckpointIdentity {
                    producer: ("producer".to_owned()).into(),
                    posture: super::super::WorthQueryAcceptedOutputCheckpointPosture::Performed,
                    source: [0x31; 32],
                    scope,
                    source_partition: partition(number),
                    producer_dependency: None,
                    idempotency_key: [0x41; 32],
                    resources: None,
                    roles: (Vec::new()).into(),
                    producer_facts: None,
                    producer_fact_wire_version: 0,
                    metadata: None,
                },
                correspondence: Arc::clone(&correspondence),
            })
            .collect();
        let recovered =
            WorthQueryRecoveredOutputs::from_records(records, metadata, &execution).unwrap();
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
    fn restored_selection_excludes_100k_unrelated_partitions() {
        unrelated_restored_selection(100_000);
    }
}
