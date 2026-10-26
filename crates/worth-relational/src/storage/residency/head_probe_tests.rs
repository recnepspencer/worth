//! Real current invariant reads never reconstruct history, including baselines.
use super::*;
use crate::storage::overlay::{CurrentPartitionAccess, OverlayStateView};
use crate::validation::engine::state_view::InvariantStateView;

#[derive(Debug)]
struct HistoryUnavailable(Arc<CountedSource>);
impl RelationalDurableColdSource for HistoryUnavailable {
    fn read_member(
        &self,
        locator: RelationalDurableMemberLocator,
        limit: u64,
    ) -> Result<Vec<u8>, Cause> {
        if locator == self.0.history {
            return Err(Cause::SourceUnavailable);
        }
        self.0.read_member(locator, limit)
    }
    fn read_commit(&self, _: PatchStreamPosition, _: u64) -> Result<Vec<u8>, Cause> {
        Err(Cause::SourceUnavailable)
    }
}

fn edition(fixture: &RegionFixture) -> crate::runtime::PartitionEdition {
    let mut cold = fixture.cold.clone();
    cold.source = Arc::new(HistoryUnavailable(fixture.source.clone()));
    crate::runtime::PartitionEdition::new(BTreeMap::from([(
        fixture.entity.partition_id,
        RelationalRegionRetention::new(RelationalRegionResidency::cold(cold), None),
    )]))
}

#[test]
fn invariant_current_rows_and_aspects_succeed_when_history_is_unavailable() {
    let fixture = region_with_history();
    let edition = edition(&fixture);
    let host = crate::test_execution_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&host);
    let mut rounds = 0;
    let answer = probe_head(&execution, &edition, |source| {
        rounds += 1;
        let view = InvariantStateView::new(source, crate::identity::data::VersionId(u64::MAX));
        (
            view.entity_metadata(fixture.entity)
                .map(|metadata| metadata.entity_id),
            view.relation_metadata(fixture.relation)
                .map(|metadata| metadata.source),
            view.entity_aspect_state(fixture.entity).cloned(),
            view.outgoing_relations_for_entity(fixture.entity),
        )
    })
    .unwrap();
    assert_eq!(answer.0, Some(fixture.entity));
    assert_eq!(answer.1, Some(fixture.entity));
    assert!(answer.2.is_some());
    assert_eq!(answer.3, vec![fixture.relation]);
    assert!(rounds > 1 && rounds < 4);
    assert_eq!(fixture.source.history_reads.load(Ordering::SeqCst), 0);
    let failure = crate::storage::overlay::PartitionAccess::demand_full_partition(
        &edition,
        &execution,
        fixture.entity.partition_id,
    )
    .unwrap_err();
    assert!(matches!(
        failure,
        RelationalResidencyDemandFailure::StorageUnavailable(_)
    ));
}

#[test]
fn head_rounds_probe_related_sources_and_an_independent_cold_before_image() {
    let fixtures = [region_with_history(), region_with_history()];
    let editions = fixtures.each_ref().map(edition);
    let staged = full_demand(&fixtures[0].cell, 1 << 24).unwrap();
    let staged = BTreeMap::from([(
        fixtures[0].entity.partition_id,
        staged.full_view().partition().clone(),
    )]);
    let overlay = OverlayStateView::new(&editions[0], &staged);
    let host = crate::test_execution_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&host);
    let id = fixtures[0].entity.partition_id;
    let mut rounds = 0;
    let answer = probe_head_sources(
        &execution,
        [&overlay as &dyn CurrentPartitionAccess, &editions[1]],
        |[visible, related]| {
            rounds += 1;
            [
                visible.get_current_partition(id).is_some(),
                visible.base_current_partition(id).is_some(),
                related.get_current_partition(id).is_some(),
            ]
        },
    )
    .unwrap();
    assert_eq!(answer, [true, true, true]);
    assert!(rounds > 1 && rounds < 4);
    assert_eq!(fixtures[1].source.history_reads.load(Ordering::SeqCst), 0);
}

#[test]
fn provisional_head_absence_never_masks_sticky_damage() {
    let fixture = region_with_history();
    fixture.source.corrupt.store(true, Ordering::SeqCst);
    let edition = edition(&fixture);
    let host = crate::test_execution_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&host);
    let attempt = || {
        probe_head(&execution, &edition, |view| {
            view.get_current_partition(fixture.entity.partition_id)
                .is_some()
        })
    };
    let first = attempt().unwrap_err();
    assert!(matches!(
        first,
        RelationalResidencyDemandFailure::StorageUnavailable(_)
    ));
    let reads = fixture.source.reads.load(Ordering::SeqCst);
    fixture.source.corrupt.store(false, Ordering::SeqCst);
    assert_eq!(attempt().unwrap_err(), first);
    assert_eq!(fixture.source.reads.load(Ordering::SeqCst), reads);
}
