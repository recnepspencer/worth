use super::*;

pub(super) fn occurrence() -> worth_runtime_world::facade::ProductBranchIncarnation {
    static OCCURRENCE: std::sync::OnceLock<worth_runtime_world::facade::ProductBranchIncarnation> =
        std::sync::OnceLock::new();
    *OCCURRENCE.get_or_init(|| {
        let world =
            crate::domain_computation::primary_graph::tests::fixture::installed_authorization_world(
                true,
            );
        let product = world
            .application
            .product_runtime()
            .admit_product_branch(world.application.product_runtime().default_branch())
            .expect("the fixture's default product occurrence is live");
        product.observation().lifecycle_incarnation()
    })
}

pub(super) fn key(producer: &str, revision: u64, tail: u8) -> WorthQueryOutputDemandKey {
    key_with_identity(producer, revision, tail, revision)
}

pub(super) fn key_with_identity(
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

pub(super) fn key_with_query_identity(
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
    WorthQueryOutputDemandKey::new(
        producer.to_owned(),
        crate::domain_computation::primary_graph::application_query::WorthQueryObservedSourceEpoch::new(
            query,
            [query_identity.wrapping_add(1); 32],
            root,
            occurrence(),
            observation_generation,
            identity,
            [query_identity.wrapping_add(2); 32],
        ),
    )
}

pub(super) fn root(slot: u8) -> worth_relational::facade::identity::EntityId {
    worth_relational::facade::identity::EntityId::new(
        worth_relational::facade::identity::PartitionId::main(),
        u64::from(slot),
        1,
    )
}

pub(super) fn record(
    occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
    state: DemandState,
    interests: usize,
) -> DemandRecord {
    DemandRecord {
        interests,
        required: false,
        product_occurrence: occurrence,
        source_scope: None,
        source_commits: Vec::new(),
        state,
        performed_source: None,
        successor_of: None,
        wake: Arc::new(DemandWake {
            generation: Mutex::new(0),
            changed: Condvar::new(),
        }),
    }
}

pub(super) fn interest(
    registry: &WorthQueryOutputDemandRegistry,
    key: WorthQueryOutputDemandKey,
    wake: Arc<DemandWake>,
) -> WorthQueryOutputDemandInterest {
    WorthQueryOutputDemandInterest {
        key,
        notifications: WorthQueryOutputDemandNotifications {
            wake: Arc::clone(&wake),
        },
        owner: registry.clone(),
    }
}
