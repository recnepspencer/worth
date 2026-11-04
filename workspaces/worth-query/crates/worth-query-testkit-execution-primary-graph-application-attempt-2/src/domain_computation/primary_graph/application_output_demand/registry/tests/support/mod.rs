use crate::domain_computation::primary_graph::application_output_demand::registry::tests::*;

pub fn occurrence() -> worth_runtime_world::facade::ProductBranchIncarnation {
    static OCCURRENCE: std::sync::OnceLock<worth_runtime_world::facade::ProductBranchIncarnation> =
        std::sync::OnceLock::new();
    *OCCURRENCE.get_or_init(|| {
        let world =
            worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world(
                true,
            );
        let product = world
            .application
            .product_runtime()
            .admit_product_branch_for_test(world.application.product_runtime().default_branch_for_test())
            .expect("the fixture's default product occurrence is live");
        product.observation_for_test().lifecycle_incarnation()
    })
}

pub fn key(producer: &str, revision: u64, tail: u8) -> WorthQueryOutputDemandKey {
    key_with_identity(producer, revision, tail, revision)
}

pub fn key_with_identity(
    producer: &str,
    observation_generation: u64,
    root_slot: u8,
    semantic_identity: u64,
) -> WorthQueryOutputDemandKey {
    key_with_query_identity(
        producer,
        observation_generation,
        root_slot,
        semantic_identity,
        1,
    )
}

pub fn key_with_query_identity(
    producer: &str,
    observation_generation: u64,
    root_slot: u8,
    semantic_identity: u64,
    query_identity: u8,
) -> WorthQueryOutputDemandKey {
    let mut identity = [0; 32];
    identity[..8].copy_from_slice(&semantic_identity.to_be_bytes());
    let query = [query_identity; 32];
    let root = root(root_slot);
    WorthQueryOutputDemandKey::new_for_test(
        "registry-output-family",
        producer.to_owned(),
        worth_query_execution::facade::application_contribution::WorthQueryProducerApplicability::new("registry-fixture", worth_query_execution::facade::application_contribution::WorthQueryProducerLifecyclePosture::Preserve),
        worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_query::observed_source::source_identity::WorthQueryObservedSourceEpoch::new_for_test(
            query,
            [query_identity.wrapping_add(1); 32],
            root,
            occurrence(),
            observation_generation,
            identity,
        ),
    )
}

pub fn root(slot: u8) -> worth_relational::facade::identity::EntityId {
    worth_relational::facade::identity::EntityId::new(
        worth_relational::facade::identity::PartitionId::main(),
        u64::from(slot),
        1,
    )
}

pub fn record(
    occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
    state: DemandState,
    interests: usize,
) -> DemandRecord {
    DemandRecord::new_for_test(
        None,
        interests,
        0,
        Vec::new(),
        0,
        Vec::new(),
        None,
        None,
        0,
        None,
        false,
        Default::default(),
        Vec::new(),
        occurrence,
        None,
        Vec::new(),
        state,
        None,
        None,
        None,
        None,
        None,
        Arc::new(DemandWake::new_for_test(Mutex::new(0), Condvar::new())),
    )
}

pub fn interest(
    registry: &WorthQueryOutputDemandRegistry,
    key: WorthQueryOutputDemandKey,
    wake: Arc<DemandWake>,
) -> WorthQueryOutputDemandInterest {
    interest_with_requirement(registry, key, wake, false)
}

pub fn required_interest(
    registry: &WorthQueryOutputDemandRegistry,
    key: WorthQueryOutputDemandKey,
    wake: Arc<DemandWake>,
) -> WorthQueryOutputDemandInterest {
    interest_with_requirement(registry, key, wake, true)
}

pub fn interest_with_requirement(
    registry: &WorthQueryOutputDemandRegistry,
    key: WorthQueryOutputDemandKey,
    wake: Arc<DemandWake>,
    requires_output: bool,
) -> WorthQueryOutputDemandInterest {
    WorthQueryOutputDemandInterest::new_for_test(
        key,
        requires_output,
        WorthQueryOutputDemandNotifications::new_for_test(Arc::clone(&wake)),
        registry.clone(),
    )
}

/// Retain the owned key exactly as prepare_new_record does, without activating work.
pub fn retain_owned_key(state: &mut DemandRegistryState, key: &WorthQueryOutputDemandKey) {
    if (*(*state.records_for_test())[key].work_membership_for_test()).is_some() {
        return;
    }
    let queue = (*state
        .required_work_queue_mut_for_test())
        .get_or_insert_with(|| Arc::new(worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::required_work::queue::RequiredWorkQueue::new()));
    let member = Arc::new(worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::required_work::RequiredWorkMembership::new_for_test(
        Arc::new(key.clone()),
        queue,
    ));
    (*(*state.records_mut_for_test())
        .get_mut(key)
        .unwrap()
        .work_membership_mut_for_test()) = Some(member);
}
