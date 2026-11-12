//! Public DATA preparation only: these bytes grant no owner or Native authority.
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use worth_execution::{ExecutionImmutableBytes, ExecutionRequest, ExecutionRequestArrayBuilder};
use worth_foundational::facade::*;
use worth_relational::facade::{
    branch::RecordedRelationalComponentReference, durability::RelationalWorldSourceDescription,
};
use worth_runtime_world::facade::*;

#[derive(Debug)]
struct ObservedDataCut {
    source: Box<dyn WorldCheckpointReadSource>,
    reads: Arc<AtomicUsize>,
}
impl WorldCheckpointReadSource for ObservedDataCut {
    fn read(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<WorldCheckpointReadView, WorldDurableMediumDenial> {
        self.source.read(request)
    }
    fn read_journal_record(
        &self,
        request: &ExecutionRequest<'_, '_>,
        journal: WorldDurableJournal,
        position: u64,
    ) -> Result<WorldCheckpointJournalRecord, WorldDurableMediumDenial> {
        assert_eq!(journal, WorldDurableJournal::Branches);
        assert_eq!(position, 4);
        self.reads.fetch_add(1, Ordering::Relaxed);
        self.source.read_journal_record(request, journal, position)
    }
    fn observe_selection(
        &self,
        _: &WorldCheckpointSubject,
    ) -> Result<Option<WorldCheckpointSelection>, WorldDurableMediumDenial> {
        unreachable!("DATA preparation never selects a checkpoint")
    }
}
fn bytes(request: &ExecutionRequest<'_, '_>, input: &[u8]) -> ExecutionImmutableBytes {
    let mut out = ExecutionRequestArrayBuilder::allocate(*request, input.len()).unwrap();
    out.extend_from_slice(input).unwrap();
    out.seal_immutable_bytes().unwrap()
}

#[test]
fn protected_home_data_prepares_every_branch_from_selected_image_and_original_tail() {
    let funding = crate::test_execution_policy::serial_request();
    let request = ExecutionRequest::serial(&funding);
    let reads = Arc::new(AtomicUsize::new(0));
    let source = Box::new(ObservedDataCut {
        source: super::artifact_source::Recorder::recorded_data_cut(
            bytes(&request, include_bytes!("branch_data/selected.world")),
            bytes(&request, include_bytes!("branch_data/tail.world")),
        ),
        reads: reads.clone(),
    });
    let binding = WorldDurableHomeBinding::from_admitted_adapter(
        DurableWorldOwnerIdentity::from_recorded(
            HomeIdentity::from_observed_bytes([7; 16]).unwrap(),
            WorldOwnerSlotIdentity::from_recorded(0),
        ),
        HomeIncarnation::initial(),
        [3; 32],
    );
    let expected = [
        include_bytes!("branch_data/primary-native.json").as_slice(),
        include_bytes!("branch_data/sibling-native.json").as_slice(),
        include_bytes!("branch_data/other-native.json").as_slice(),
    ]
    .map(|json| {
        RelationalWorldSourceDescription::Recorded(
            RecordedRelationalComponentReference::from_json_in(&request, json).unwrap(),
        )
    });
    let prepared = PreparedWorldHomeData::from_home_source(binding, source, &request).unwrap();
    let catalog = prepared.native_selection().unwrap();
    assert_eq!(catalog.heads().len(), 3);
    for source in &expected {
        assert!(catalog.heads().contains(source));
    }
    let selected = catalog.selected_checkpoint().unwrap();
    assert_eq!(selected.heads().len(), 2);
    assert!(selected.heads().contains(&expected[0]));
    assert!(selected.heads().contains(&expected[1]));
    assert!(!selected.heads().contains(&expected[2]));
    assert_eq!(reads.load(Ordering::Relaxed), 1);
}
