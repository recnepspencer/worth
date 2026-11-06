//! Exact restored-output locators; reconstructed storage keeps its bootstrap payer.
use super::WorthQueryReadmittedAcceptedOutput;
use {crate::domain_computation::primary_graph::application_checkpoint::CheckpointDecodeDenial, crate::domain_computation::primary_graph::application_checkpoint::CheckpointMetadataFunding, crate::domain_computation::primary_graph::application_checkpoint::DecodedApplicationCheckpoint, crate::domain_computation::graph_layout::denial::WorthQueryPrimaryGraphInstallationDenial as Denial, crate::domain_computation::vocabulary::authorization::operation_scope_binding::WorthQueryOperationScopeEntityBinding};

#[derive(Default)]
pub struct WorthQueryRecoveredOutputs {
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
                crate::domain_computation::primary_graph::vocabulary::application_attempt::output_correspondence::WorthQueryApplicationOutputCorrespondence,
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
    pub fn iter(&self) -> impl Iterator<Item = &WorthQueryReadmittedAcceptedOutput> {
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

mod court_doors_for_test;
