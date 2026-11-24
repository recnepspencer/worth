//! Real current invariant reads never reconstruct history, including baselines.
use super::*;
use worth_relational_mutation::internal_port::validation::engine::state_view::InvariantStateView;
use worth_relational_storage::internal_port::storage::overlay::access::OverlayStateView;
use worth_relational_storage::internal_port::storage::overlay::current_access::CurrentPartitionAccess;

#[derive(Debug)]
struct HistoryUnavailable(Arc<CountedSource>);
impl RelationalDurableColdSource for HistoryUnavailable {
    fn retain_members_for_request(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        members: &[RelationalDurableMemberLocator],
        issuer: std::sync::Weak<dyn RelationalDurableColdSource>,
    ) -> Result<
        worth_relational_storage::internal_port::durability::medium::member_custody::RelationalDurableMemberCustody,
        worth_relational_storage::internal_port::durability::medium::read_protection::refusal::RelationalDurableOwnedReadRefusal,
    >{
        self.0.retain_members_for_request(request, members, issuer)
    }

    fn read_member_owned(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        original: &(dyn std::any::Any + Send + Sync),
        member: RelationalDurableMemberLocator,
        limit: u64,
    ) -> Result<
        worth_execution::ExecutionImmutableBytes,
        worth_relational_storage::internal_port::durability::medium::read_protection::refusal::RelationalDurableOwnedReadRefusal,
    >{
        if member == self.0.history {
            return Err(
                worth_relational_storage::internal_port::durability::medium::read_protection::refusal::RelationalDurableOwnedReadRefusal::Storage(
                    Cause::SourceUnavailable,
                ),
            );
        }
        self.0.read_member_owned(request, original, member, limit)
    }

    fn validate_member_custody_retirement(
        &self,
        original: &(dyn std::any::Any + Send + Sync),
    ) -> Result<(), worth_relational_storage::internal_port::durability::medium::member_custody::retirement::RelationalDurableMemberRetirementRefusal>{
        self.0.validate_member_custody_retirement(original)
    }

    fn advance_member_custody_retirement(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        original: &mut (dyn std::any::Any + Send + Sync),
    ) -> worth_relational_storage::internal_port::durability::medium::member_custody::retirement::RelationalDurableMemberRetirementObservation{
        self.0.advance_member_custody_retirement(request, original)
    }

    fn retain_members(
        &self,
        members: &[RelationalDurableMemberLocator],
    ) -> Result<worth_relational_storage::internal_port::durability::medium::member_custody::RelationalDurableMemberCustody, Cause>{
        self.0.retain_members(members)
    }

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

fn edition(
    fixture: &RegionFixture,
) -> worth_relational_storage::internal_port::runtime::state::subsystems::storage::partition_edition::PartitionEdition{
    let mut cold = fixture.cold.clone();
    cold.source = Some(Arc::new(HistoryUnavailable(fixture.source.clone())));
    worth_relational_storage::internal_port::runtime::state::subsystems::storage::partition_edition::PartitionEdition::new(
        BTreeMap::from([(
            fixture.entity.partition_id,
            RelationalRegionRetention::new(RelationalRegionResidency::cold(cold).unwrap(), None),
        )])
        .into_iter()
        .collect(),
    )
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
    let host = worth_harness::facade::execution_fixture::serial_request();
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
    let host = worth_harness::facade::execution_fixture::serial_request();
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

#[test]
fn invariant_current_rows_and_aspects_succeed_when_history_is_unavailable() {
    let fixture = region_with_history();
    let edition = edition(&fixture);
    let host = worth_harness::facade::execution_fixture::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&host);
    let mut rounds = 0;
    let answer = probe_head(&execution, &edition, |source| {
        rounds += 1;
        let view = InvariantStateView::new(
            source,
            worth_relational_contracts::internal_port::identity::data::primitives::VersionId(
                u64::MAX,
            ),
        );
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
    let failure = edition
        .demand_partition(&execution, fixture.entity.partition_id)
        .unwrap_err();
    assert!(matches!(
        failure,
        RelationalResidencyDemandFailure::StorageUnavailable(_)
    ));
}
