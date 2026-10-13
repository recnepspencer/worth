use crate::facade::identity::PartitionId;
use crate::facade::runtime::RelationalRuntime;
use crate::tests::support::read_entity_name;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct VisibleRelationSummary {
    pub(crate) partition: PartitionId,
    pub(crate) source_name: String,
    pub(crate) target_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct VisibleTruthSummary {
    pub(crate) entity_names: Vec<String>,
    pub(crate) relations: Vec<VisibleRelationSummary>,
}

impl VisibleTruthSummary {
    pub(crate) fn capture(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &RelationalRuntime,
    ) -> Self {
        let snapshot = runtime.visibility_authority().snapshot();
        let read = runtime
            .read_truth()
            .read_snapshot(&snapshot, execution)
            .expect("read request remains live")
            .unwrap();

        let mut entity_names = read
            .entities()
            .iter()
            .filter_map(read_entity_name)
            .collect::<Vec<_>>();
        entity_names.sort();

        let mut relations = read
            .relations()
            .iter()
            .filter_map(|relation| {
                let source = read
                    .get_entity(relation.source)
                    .and_then(read_entity_name)?;
                let target = read
                    .get_entity(relation.target)
                    .and_then(read_entity_name)?;
                Some(VisibleRelationSummary {
                    partition: relation.relation_id.partition_id,
                    source_name: source,
                    target_name: target,
                })
            })
            .collect::<Vec<_>>();
        relations.sort();

        drop(read);
        assert!(runtime
            .visibility_authority()
            .release_snapshot(&snapshot)
            .is_ok());

        Self {
            entity_names,
            relations,
        }
    }
}
