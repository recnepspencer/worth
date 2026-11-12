//! Reconstruct graph storage from declarations, retaining only fresh installation bindings.
use super::SignalGraph;
use crate::data::conditional_execution::InstalledSignalConditionalContract;
use crate::data::dependency::{CanonicalDependencies, DependencyEdge};

/// Reconstructed declarations and freshly installed conditional bindings.
/// This packet carries no admitted owner or branch-basis authority.
pub struct SignalAuthoredGraphReadmission {
    graph: Box<SignalGraph>,
    conditional_contracts: Vec<InstalledSignalConditionalContract>,
}
impl SignalAuthoredGraphReadmission {
    pub fn into_parts(self) -> (Box<SignalGraph>, Vec<InstalledSignalConditionalContract>) {
        (self.graph, self.conditional_contracts)
    }
}
use crate::data::node::NodeEntry;
use crate::durability::{SignalAuthoredImage, SignalAuthoredImageDenial as Denial};

impl SignalGraph {
    pub(crate) fn validate_authored_comparator(
        &self,
        policy: &crate::data::comparator::VersionComparatorPolicy,
    ) -> Result<(), Denial> {
        use crate::data::comparator::{InstalledSignalComparatorRole, VersionComparatorPolicy};
        use crate::data::conditional_execution::{
            SignalConditionalArtifactReuse, SignalConditionalVersionComparator,
        };
        let identity = match policy {
            VersionComparatorPolicy::Installed { identity } => identity,
            VersionComparatorPolicy::Exact
            | VersionComparatorPolicy::Tolerance { .. }
            | VersionComparatorPolicy::OutputIdentity
            | VersionComparatorPolicy::Custom { .. } => return Ok(()),
        };
        if identity.graph_instance_id() != self.instance_id {
            return Err(Denial::ForeignConditionalBinding);
        }
        let node = identity.node();
        if !self.is_alive(node) {
            return Err(Denial::ConditionalNodeMissing);
        }
        let definition = self.arena.definitions[node.index() as usize]
            .conditional_definition
            .as_ref()
            .ok_or(Denial::ConditionalDefinitionMissing)?;
        let declared = match identity.role() {
            InstalledSignalComparatorRole::DependencyVersion => {
                definition.dependency_comparator
                    == SignalConditionalVersionComparator::RuntimeResolved
            }
            InstalledSignalComparatorRole::OutputEquivalence => {
                definition.output_comparator == SignalConditionalVersionComparator::RuntimeResolved
            }
            InstalledSignalComparatorRole::ArtifactReuse => {
                definition.artifact_reuse == SignalConditionalArtifactReuse::RuntimeResolved
            }
        };
        if !declared {
            return Err(Denial::ForeignConditionalBinding);
        }
        Ok(())
    }
    /// Source reconstruction. The owner must finish its Bridge definition
    /// admission before sealing any live service over this staged graph.
    pub fn readmit_authored_image(
        self: Box<Self>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        image: &SignalAuthoredImage,
    ) -> Result<SignalAuthoredGraphReadmission, Denial> {
        super::SignalGraphInstallationBindings::consume(self).readmit(execution, image)
    }
    pub(in crate::data::graph) fn readmit_authored_rows(
        mut cold: Box<Self>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        image: &SignalAuthoredImage,
    ) -> Result<SignalAuthoredGraphReadmission, Denial> {
        crate::durability::authored_image::admit_work(execution, image.as_bytes().len())?;
        let rows = image.read_graph_rows(cold.instance_id)?;
        for row in rows.slots {
            let (definition, hot, warm, diagnostic) = match row.definition {
                Some(definition) => {
                    let (_, hot, warm, diagnostic) = NodeEntry::new().into_storage_parts();
                    cold.arena.active_nodes += 1;
                    (definition, Some(hot), warm, diagnostic)
                }
                None => (Default::default(), None, Default::default(), None),
            };
            cold.arena.nodes.push_back(row.allocation);
            cold.arena.definitions.push_back(definition);
            cold.arena.hot.push_back(hot);
            cold.arena.warm.push_back(warm);
            cold.arena.cold.push_back(diagnostic);
        }
        for index in rows.free_list {
            cold.arena.free_list.push_back(index);
            cold.arena.free_slots.mark(index as usize);
        }
        cold.rebuild_authored_topology()
            .map_err(|cause| Denial::Topology(Box::new(cause)))?;
        let mut conditional_contracts = Vec::new();
        for node in cold.live_node_ids() {
            let Some(source) = cold.arena.definitions[node.index() as usize]
                .conditional_definition
                .take()
            else {
                continue;
            };
            let definition = std::sync::Arc::try_unwrap(source)
                .expect("a freshly decoded source definition has sole custody");
            let owner = cold
                .aspect_lowering_owner
                .clone()
                .ok_or(Denial::ConditionalLoweringOwnerMissing)?;
            let capability = match cold.admit_installed_node(node) {
                worth_proof::TransitionOutcome::Success(capability) => capability,
                worth_proof::TransitionOutcome::Denied(_) => {
                    return Err(Denial::ConditionalNodeMissing)
                }
                worth_proof::TransitionOutcome::Deferred(never)
                | worth_proof::TransitionOutcome::Stale(never)
                | worth_proof::TransitionOutcome::RebindRequired(never)
                | worth_proof::TransitionOutcome::Failed(never) => match never {},
            };
            conditional_contracts.push(
                cold.install_conditional_contract(&owner, capability, definition)
                    .map_err(Denial::ConditionalInstallation)?,
            );
        }
        Ok(SignalAuthoredGraphReadmission {
            graph: cold,
            conditional_contracts,
        })
    }
    fn rebuild_authored_topology(&mut self) -> Result<(), crate::data::error::SignalError> {
        let declarations = self
            .live_node_ids()
            .into_iter()
            .map(|node| {
                (
                    node,
                    self.arena.definitions[node.index() as usize]
                        .declared_dependencies
                        .clone()
                        .unwrap_or_else(|| CanonicalDependencies::new([])),
                )
            })
            .collect::<Vec<_>>();
        // Use the same complete-set validation as ordinary authored edits before
        // installing any adjacency or derived index into the reconstructed graph.
        self.validate_declared_edits(&declarations)?;
        for (node, declarations) in declarations {
            let edges = CanonicalDependencies::new(declarations.as_slice().iter().map(|edge| {
                match edge.scope_ref().cloned() {
                    Some(scope) => {
                        let interned = self
                            .observation
                            .partition_interner
                            .intern_subscription(&scope);
                        DependencyEdge::with_scope(edge.source(), edge.aspect(), scope, interned)
                    }
                    None => DependencyEdge::new(edge.source(), edge.aspect()),
                }
            }));
            let id = self
                .topology
                .dependency_edges
                .insert_from_slice(edges.as_slice());
            self.set_dependencies_id_direct(node, id)?;
        }
        self.rebuild_subscriber_index_from_dependencies()?;
        self.rebuild_declared_dependents();
        self.rebuild_reverse_subscription_index_from_dependencies()
    }
}
