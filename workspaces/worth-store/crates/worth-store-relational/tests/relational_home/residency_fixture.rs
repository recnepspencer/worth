//! A partitioned, history-bearing script executed by the oracle's memory runtime.
use super::{batches, schema_transition, workload};
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    history::BranchId,
    identity::PartitionId,
    runtime::RelationalRuntime,
    transactions::{CreateIntent, MutationIntent, RecordRef},
};

pub(super) struct Fixture {
    pub reference: RelationalRuntime,
    pub steps: Vec<workload::Step>,
    pub records: Vec<RecordRef>,
}

impl Fixture {
    pub fn new(execution: &ExecutionRequest<'_, '_>) -> Self {
        let reference = workload::runtime_builder(schema_transition::SOURCE).build();
        let branch = BranchId("main".into());
        let mut steps = Vec::new();
        let mut records = Vec::new();
        for partition in 1..=24 {
            let mut batch = batches::create(partition as usize);
            let MutationIntent::Create(CreateIntent::Entity(spec)) = &mut batch.intents[0] else {
                panic!("entity fixture")
            };
            spec.partition_id = PartitionId(partition);
            let outcome = workload::commit(&reference, execution, &branch, batch.clone()).unwrap();
            records.extend(outcome.changed_records.iter().cloned());
            steps.push(workload::Step::Commit {
                branch: branch.clone(),
                batch,
            });
        }
        for round in 0..3 {
            for record in &records {
                let RecordRef::Entity(entity) = *record else {
                    panic!("entity fixture")
                };
                let batch = batches::update(
                    entity,
                    100 + round * 24 + entity.partition_id.as_u32() as usize,
                );
                workload::commit(&reference, execution, &branch, batch.clone()).unwrap();
                steps.push(workload::Step::Commit {
                    branch: branch.clone(),
                    batch,
                });
            }
        }
        steps.push(workload::Step::Checkpoint);
        Self {
            reference,
            steps,
            records,
        }
    }

    pub fn assert_reads(
        &self,
        home: &worth_store_relational::RelationalStoreHome,
        execution: &ExecutionRequest<'_, '_>,
    ) {
        let reopened = home.cold_storage_reads();
        assert_eq!(
            reopened.member_reads, 0,
            "killed-home reopen loads only the manifest"
        );
        assert!(
            reopened.commit_reads < 8,
            "reopen must not read the 96-commit history"
        );
        let point = &self.records[..1];
        let expected_point = workload::states_of(&self.reference, point);
        assert_eq!(workload::states_of(home.runtime(), point), expected_point);
        let partial = home.cold_storage_reads();
        assert_eq!(
            partial.commit_reads, reopened.commit_reads,
            "a point read opens no historical envelope"
        );
        assert!(
            partial.member_reads < 12,
            "point demand must leave the other 23 partitions cold: {partial:?}"
        );
        let expected = workload::states_of(&self.reference, &self.records);
        assert_eq!(
            workload::states_of(home.runtime(), &self.records),
            expected,
            "partly warm reads"
        );
        let warm = home.cold_storage_reads();
        assert_eq!(
            workload::states_of(home.runtime(), &self.records),
            expected,
            "warm reads"
        );
        assert_eq!(home.cold_storage_reads(), warm);
        home.runtime()
            .durability_authority()
            .checkpoint(execution)
            .unwrap();
        assert_eq!(
            workload::states_of(home.runtime(), &self.records),
            expected,
            "Full reads"
        );
        assert_eq!(
            workload::artifacts_of(home.runtime()),
            workload::artifacts_of(&self.reference)
        );
    }
}
