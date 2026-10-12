use crate::memory_workspace::WorthQueryEntityIdentity;
use crate::runtime::tests::graph_read_access::support::aspect_touch as touch;
use crate::runtime::WorthQueryWorkspace;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HostileGraphFixtureSummary {
    user_count: usize,
    active_user_count: usize,
    relation_edge_count: usize,
    branching_factor: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct HostileFrontierGraphSpec {
    active_user_count: usize,
    inactive_decoy_count: usize,
    branching_factor: usize,
}

impl HostileFrontierGraphSpec {
    fn broad_branching_with_inactive_decoys() -> Self {
        Self {
            active_user_count: 48,
            inactive_decoy_count: 8,
            branching_factor: 6,
        }
    }

    fn total_user_count(&self) -> usize {
        self.active_user_count + self.inactive_decoy_count
    }
}

impl HostileGraphFixtureSummary {
    pub fn user_count(&self) -> usize {
        self.user_count
    }

    pub fn active_user_count(&self) -> usize {
        self.active_user_count
    }

    pub fn relation_edge_count(&self) -> usize {
        self.relation_edge_count
    }

    pub fn branching_factor(&self) -> usize {
        self.branching_factor
    }
}

pub fn seed_hostile_frontier_graph(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    workspace: &mut WorthQueryWorkspace,
    prefix: &str,
) -> HostileGraphFixtureSummary {
    let spec = HostileFrontierGraphSpec::broad_branching_with_inactive_decoys();
    let mut active_identities = Vec::with_capacity(spec.active_user_count);
    for index in 0..spec.active_user_count {
        let receipt = workspace
            .insert(request_execution, "user", |user| {
                user.set_aspect(touch("identity.id"), authored_text(user_id(prefix, index)))
                    .set_aspect(touch("status.value"), authored_text("active"))
                    .set_aspect(
                        touch("profile.display_name"),
                        authored_text(format!("User {index:03}")),
                    )
            })
            .expect("hostile active user should insert");
        active_identities.push(receipt.deltas()[0].entity_identity().clone());
    }
    for index in spec.active_user_count..spec.total_user_count() {
        workspace
            .insert(request_execution, "user", |user| {
                user.set_aspect(touch("identity.id"), authored_text(user_id(prefix, index)))
                    .set_aspect(touch("status.value"), authored_text("inactive"))
                    .set_aspect(
                        touch("profile.display_name"),
                        authored_text(format!("Decoy {index:03}")),
                    )
            })
            .expect("hostile decoy user should insert");
    }
    let relation_edge_count = seed_relation_edges(
        request_execution,
        workspace,
        prefix,
        "manager",
        &active_identities,
        spec.branching_factor,
    ) + seed_relation_edges(
        request_execution,
        workspace,
        prefix,
        "mentor",
        &active_identities,
        spec.branching_factor,
    );
    HostileGraphFixtureSummary {
        user_count: spec.total_user_count(),
        active_user_count: spec.active_user_count,
        relation_edge_count,
        branching_factor: spec.branching_factor,
    }
}

fn seed_relation_edges(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    workspace: &mut WorthQueryWorkspace,
    prefix: &str,
    relation: &str,
    active_identities: &[WorthQueryEntityIdentity],
    branching_factor: usize,
) -> usize {
    let mut edge_count = 0;
    for source_index in 0..active_identities.len() {
        for branch in 1..=branching_factor {
            let target_index = (source_index + branch) % active_identities.len();
            workspace
                .insert(request_execution, relation, |edge| {
                    edge.set_aspect(
                        touch("identity.id"),
                        authored_text(format!("{prefix}-{relation}-{source_index}-{target_index}")),
                    )
                    .set_aspect(
                        touch("source.id"),
                        authored_entity_ref(&active_identities[source_index]),
                    )
                    .set_aspect(
                        touch("target.id"),
                        authored_entity_ref(&active_identities[target_index]),
                    )
                })
                .expect("hostile relation edge should insert");
            edge_count += 1;
        }
    }
    edge_count
}

fn user_id(prefix: &str, index: usize) -> String {
    format!("{prefix}-{index}")
}

fn authored_text(value: impl Into<String>) -> crate::runtime::WorthQueryAuthoredAspectValue {
    crate::runtime::WorthQueryAuthoredAspectValue::string(value)
}

fn authored_entity_ref(
    identity: &WorthQueryEntityIdentity,
) -> crate::runtime::WorthQueryAuthoredAspectValue {
    crate::runtime::WorthQueryAuthoredAspectValue::native(
        crate::runtime::tests::support::test_native_entity_ref_value(identity),
    )
}
