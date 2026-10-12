use worth_query_host::facade::runtime;

pub(super) fn world_resources(
    commits: u64,
    metadata_bytes: u64,
    pins: u64,
) -> runtime::WorthQueryProductWorldResources {
    world_resources_with_work(commits, metadata_bytes, pins, 8_000_000)
}

pub(super) const WORKFLOW_SCALE_WIDTH: u64 = 200_000;

pub const WORKFLOW_DEFINITION_BYTES: u32 = 16 * 1024 * 1024;

pub(super) fn workflow_scale_resources(
    nodes: u16,
    connections: u16,
) -> runtime::WorthQueryProductWorldResources {
    let work = workflow_scale_work(nodes, connections);
    world_resources_with_work(1_024, 256 * 1024 * 1024, 2_048, work)
}

// The published node schema has four required and seventeen optional fields;
// the definition and connection schemas are narrower. Every relation can have
// one endpoint rule and six cardinality rules. Publication creates seven link
// families: current, lineage, node, connection, source, target and start.
fn workflow_scale_work(nodes: u16, connections: u16) -> u64 {
    const NODE_FIELDS: u64 = 4 + 17;
    const RELATION_RULES: u64 = 1 + 6;
    const PUBLICATION_LINK_FAMILIES: u64 = 7;
    const HEADROOM: u64 = 2;
    // Two definition records and the seed's mapping, principal and document.
    let records = u64::from(nodes) + u64::from(connections) + 2 + 3;
    // A node's ownership; each connection's ownership and two endpoints;
    // current-definition, lineage-definition and definition-start.
    let links = u64::from(nodes) + 3 * u64::from(connections) + 3;
    let levels = |count: u64| u64::from(u64::BITS - count.max(1).leading_zeros());
    let field_visits = 2 * NODE_FIELDS * records;
    let rule_visits = 4 * RELATION_RULES * links;
    let key_visits =
        PUBLICATION_LINK_FAMILIES * (records * (levels(records) + 2) + 2 * links * levels(records));
    // Lifecycle/aspect, field/index, then three adjacency touches per link.
    let touches = (2 + 2 * NODE_FIELDS) * records + 3 * links;
    let ordering = touches * levels(touches);
    // Operand preparation and comparison encoding, at one unit per 64 bytes.
    let encoding = 2 * u64::from(WORKFLOW_DEFINITION_BYTES).div_ceil(64);
    HEADROOM * (field_visits + rule_visits + key_visits + ordering + encoding)
}

fn world_resources_with_work(
    commits: u64,
    metadata_bytes: u64,
    pins: u64,
    work: u64,
) -> runtime::WorthQueryProductWorldResources {
    // Bounds this fixture World's serial request memory and deterministic work.
    let execution_policy = worth_foundational::ExecutionRequestPolicy::new(
        worth_foundational::ExecutionPosture::Serial,
        worth_foundational::DeterminismContract::CanonicalBitwise,
        worth_foundational::ExecutionBudget::new(
            std::num::NonZeroUsize::MIN,
            64 * 1024 * 1024,
            work,
        ),
    );
    runtime::WorthQueryProductWorldResources::install(
        runtime::RuntimeWorldBudgetInstallation {
            branches: runtime::RuntimeWorldBranchBudgetInstallation {
                live_product_branches: 32,
            },
            history: runtime::RuntimeWorldHistoryBudgetInstallation {
                retained_composite_commits: commits,
                history_metadata_bytes: metadata_bytes,
            },
            observations: runtime::RuntimeWorldObservationBudgetInstallation {
                active_observations: 128,
            },
            publication: runtime::RuntimeWorldPublicationBudgetInstallation {
                active_publication_attempts: 32,
            },
            recovery: runtime::RuntimeWorldRecoveryBudgetInstallation {
                retained_product_unpublished_records: 32,
                retained_partial_metadata_bytes: 4 * 1024 * 1024,
            },
            retention: runtime::RuntimeWorldRetentionBudgetInstallation {
                unique_exact_component_pins: pins,
                in_flight_pin_acquisition_reservations: 64,
            },
            custody: runtime::RuntimeWorldCustodyBudgetInstallation {
                owner_created_component_custody_records: 64,
            },
        },
        runtime::WorthQueryProductWorldClock::start(),
        runtime::WorthQueryInvalidationResources::install(
            runtime::WorthQueryInvalidationResourceInstallation::bounded(
                1_000_000,
                64 * 1_024 * 1_024,
                128 * 1_024 * 1_024,
                128,
            ),
        )
        .expect("the Query invalidation installation is valid"),
        execution_policy,
    )
    .expect("the document-retention World resources are valid")
}
