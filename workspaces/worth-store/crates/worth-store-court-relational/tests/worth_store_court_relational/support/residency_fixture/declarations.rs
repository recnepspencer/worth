//! A partitioned, history-bearing script executed by the oracle's memory runtime.
pub(crate) use {crate::batches, crate::schema_transition, crate::workload};
pub(crate) use {worth_execution::ExecutionRequest};
pub(crate) use {worth_relational::facade::history::BranchId, worth_relational::facade::identity::PartitionId, worth_relational::facade::runtime::RelationalRuntime, worth_relational::facade::transactions::CreateIntent, worth_relational::facade::transactions::MutationIntent, worth_relational::facade::transactions::RecordRef};

pub(crate) struct Fixture {
    pub(crate) reference: RelationalRuntime,
    pub(crate) steps: Vec<crate::workload::Step>,
    pub(crate) records: Vec<RecordRef>,
}

impl Fixture {
    pub(crate) fn new(execution: &ExecutionRequest<'_, '_>) -> Self {
        let reference = crate::workload::runtime_builder(crate::schema_transition::SOURCE)
            .build()
            .expect("valid native reference runtime configuration");
        let branch = BranchId("main".into());
        let mut steps = Vec::new();
        let mut records = Vec::new();
        for partition in 1..=24 {
            let mut batch = crate::batches::create(partition as usize);
            let worth_relational::facade::transactions::MutationIntent::Create(worth_relational::facade::transactions::CreateIntent::Entity(spec)) = &mut batch.intents[0] else {
                panic!("entity fixture")
            };
            spec.partition_id = PartitionId(partition);
            let outcome = crate::workload::commit(&reference, execution, &branch, batch.clone()).unwrap();
            records.extend(outcome.changed_records.iter().cloned());
            steps.push(crate::workload::Step::Commit {
                branch: branch.clone(),
                batch,
            });
        }
        for round in 0..3 {
            for record in &records {
                let worth_relational::facade::transactions::RecordRef::Entity(entity) = *record else {
                    panic!("entity fixture")
                };
                let batch = crate::batches::update(
                    entity,
                    100 + round * 24 + entity.partition_id.as_u32() as usize,
                );
                crate::workload::commit(&reference, execution, &branch, batch.clone()).unwrap();
                steps.push(crate::workload::Step::Commit {
                    branch: branch.clone(),
                    batch,
                });
            }
        }
        steps.push(crate::workload::Step::Checkpoint);
        Self {
            reference,
            steps,
            records,
        }
    }

    pub(crate) fn assert_reads(
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
        let expected_point = crate::workload::states_of(&self.reference, point);
        assert_eq!(crate::workload::states_of(home.runtime(), point), expected_point);
        let partial = home.cold_storage_reads();
        assert_eq!(
            partial.commit_reads, reopened.commit_reads,
            "a point read opens no historical envelope"
        );
        assert!(
            partial.member_reads < 12,
            "point demand must leave the other 23 partitions cold: {partial:?}"
        );
        let expected = crate::workload::states_of(&self.reference, &self.records);
        assert_eq!(
            crate::workload::states_of(home.runtime(), &self.records),
            expected,
            "partly warm reads"
        );
        let warm = home.cold_storage_reads();
        assert_eq!(
            crate::workload::states_of(home.runtime(), &self.records),
            expected,
            "warm reads"
        );
        assert_eq!(home.cold_storage_reads(), warm);
        home.runtime()
            .durability_authority()
            .checkpoint(execution)
            .unwrap();
        assert_eq!(
            crate::workload::states_of(home.runtime(), &self.records),
            expected,
            "Full reads"
        );
        assert_eq!(
            crate::workload::artifacts_of(home.runtime()),
            crate::workload::artifacts_of(&self.reference)
        );
    }
}