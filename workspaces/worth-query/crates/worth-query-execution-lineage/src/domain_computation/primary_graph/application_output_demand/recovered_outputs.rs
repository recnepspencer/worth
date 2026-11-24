//! Exact restored-output locators; reconstructed storage keeps its bootstrap payer.
use super::WorthQueryReadmittedAcceptedOutput;
use crate::domain_computation::{
    authorization::WorthQueryOperationScopeEntityBinding,
    primary_graph::application_checkpoint::{CheckpointDecodeDenial, CheckpointMetadataFunding},
};

#[derive(Default)]
pub struct WorthQueryRecoveredOutputs {
    checkpoint: RecoveredOutputRecords,
    journal: std::sync::OnceLock<RecoveredOutputRecords>,
}
#[derive(Default)]
struct RecoveredOutputRecords {
    records: Vec<WorthQueryReadmittedAcceptedOutput>,
    by_source_partition: Vec<usize>,
    // Declared last: every retained row/string/role/wire and index drops first.
    _metadata: Option<CheckpointMetadataFunding>,
}
impl WorthQueryRecoveredOutputs {
    /// Journal selectors are a rebuilt cache. After World seal they must stop
    /// retaining the old product image before Native closes its read custody.
    pub fn retire_home_observations(
        &mut self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), super::super::WorthQueryHomeRetirementDenial> {
        for record in self.checkpoint.records.iter_mut().chain(
            self.journal
                .get_mut()
                .into_iter()
                .flat_map(|rows| rows.records.iter_mut()),
        ) {
            request
                .consult()
                .map_err(super::super::WorthQueryHomeRetirementDenial::Execution)?;
            record.observation = None;
        }
        Ok(())
    }
    pub fn readmit_journal(
        &self,
        records: Vec<WorthQueryReadmittedAcceptedOutput>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), CheckpointDecodeDenial> {
        if self.journal.get().is_some() {
            return Ok(());
        }
        let index = RecoveredOutputRecords::from_records(
            records,
            CheckpointMetadataFunding::new(execution)?,
            execution,
        )?;
        self.journal
            .set(index)
            .map_err(|_| "accepted output journal already admitted".to_owned().into())
    }
    pub fn iter(&self) -> impl Iterator<Item = &WorthQueryReadmittedAcceptedOutput> {
        self.checkpoint.records.iter().chain(
            self.journal
                .get()
                .into_iter()
                .flat_map(|rows| rows.records.iter()),
        )
    }
    pub fn matching_source_partition(
        &self,
        scope: WorthQueryOperationScopeEntityBinding,
        partition: [u8; 32],
    ) -> impl Iterator<Item = &WorthQueryReadmittedAcceptedOutput> {
        self.journal
            .get()
            .into_iter()
            .flat_map(move |rows| rows.matching_source_partition(scope, partition))
            .chain(self.checkpoint.matching_source_partition(scope, partition))
    }
}
impl RecoveredOutputRecords {
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
    pub fn matching_source_partition(
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
