use std::collections::{HashMap, HashSet};

use worth_foundational::facade::{
    AspectContractRevision, AspectIdentity, AspectKey, AuthoritativeAspectChangeKind,
    CanonicalFieldPath,
};
use worth_runtime_bridge::facade::{BridgeSemanticAspectChange, BridgeSemanticAspectChangeBreadth};

use super::dependency_source::WorthQuerySemanticAspectDependencySource as Source;
use super::workflow_consequence_index::{
    conditional_output_mask, dependency_stage_identity, propagate_stage_consequences, role_bit,
    roles_from_mask,
};
use super::{WorthQueryCompiledSemanticAspectDependency, WorthQuerySemanticDependencyRole};

type NativeContractKey = (AspectKey, AspectIdentity, AspectContractRevision);
type NativeFieldKey = (AspectKey, AspectIdentity, AspectContractRevision);

#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
struct IndexedRoleEdge {
    dependency_ordinal: usize,
    role: WorthQuerySemanticDependencyRole,
}

pub(super) struct WorthQuerySemanticImpactIndex {
    native_contract: HashMap<NativeContractKey, Vec<WorthQuerySemanticDependencyRole>>,
    native_whole: HashMap<NativeContractKey, Vec<WorthQuerySemanticDependencyRole>>,
    native_field: HashMap<
        NativeFieldKey,
        crate::canonical_field_path_overlap_index::WorthQueryCanonicalPathOverlapIndex<
            IndexedRoleEdge,
        >,
    >,
    collection_aspect: HashMap<AspectKey, Vec<WorthQuerySemanticDependencyRole>>,
    collection_field: HashMap<
        AspectKey,
        crate::canonical_field_path_overlap_index::WorthQueryCanonicalPathOverlapIndex<
            IndexedRoleEdge,
        >,
    >,
    conditional: HashMap<
        (
            worth_query_installation::facade::WorthQueryConditionalNodeLocation,
            usize,
        ),
        u16,
    >,
    structural_membership: bool,
    window_on_ordering: bool,
    workflow_effect_receipts: HashSet<String>,
    mask_propagation_edges: usize,
}

pub(in crate::domain_installation::dependency_impact) struct WorthQueryIndexedImpact {
    pub(in crate::domain_installation::dependency_impact) roles:
        Vec<WorthQuerySemanticDependencyRole>,
    pub(in crate::domain_installation::dependency_impact) lookups: usize,
}

impl WorthQuerySemanticImpactIndex {
    pub(super) fn compile(dependencies: &[WorthQueryCompiledSemanticAspectDependency]) -> Self {
        let mut compilation = ImpactIndexCompilation::new();
        for (dependency_ordinal, dependency) in dependencies.iter().enumerate() {
            compilation.index_dependency(dependency_ordinal, dependency);
        }
        compilation.finish()
    }

    pub(super) fn semantic_roles(
        &self,
        change: &BridgeSemanticAspectChange,
    ) -> WorthQueryIndexedImpact {
        let contract_key = (
            change.aspect_key().clone(),
            change.aspect_identity(),
            change.contract_revision(),
        );
        let whole_change = change.effective_breadth()
            != BridgeSemanticAspectChangeBreadth::ExactField
            || matches!(
                change.kind(),
                AuthoritativeAspectChangeKind::WholeAspectSet
                    | AuthoritativeAspectChangeKind::WholeAspectClear
            );
        let mut roles = Vec::new();
        let mut lookups = 0;
        if whole_change {
            extend(
                &mut roles,
                self.native_contract.get(&contract_key),
                &mut lookups,
            );
            extend(
                &mut roles,
                self.collection_aspect.get(change.aspect_key()),
                &mut lookups,
            );
        } else if let Some(path) = change.effective_field_path() {
            extend(
                &mut roles,
                self.native_whole.get(&contract_key),
                &mut lookups,
            );
            extend_overlapping(
                &mut roles,
                self.native_field.get(&contract_key),
                path,
                &mut lookups,
            );
            extend_overlapping(
                &mut roles,
                self.collection_field.get(change.aspect_key()),
                path,
                &mut lookups,
            );
        }
        if self.window_on_ordering && roles.contains(&WorthQuerySemanticDependencyRole::Ordering) {
            roles.push(WorthQuerySemanticDependencyRole::WindowBoundary);
        }
        WorthQueryIndexedImpact { roles, lookups }
    }

    pub(super) fn contains_conditional(
        &self,
        location: &worth_query_installation::facade::WorthQueryConditionalNodeLocation,
        ordinal: usize,
    ) -> bool {
        self.conditional.contains_key(&(location.clone(), ordinal))
    }

    pub(super) fn conditional_consequence_roles(
        &self,
        location: &worth_query_installation::facade::WorthQueryConditionalNodeLocation,
        ordinal: usize,
    ) -> Option<Vec<WorthQuerySemanticDependencyRole>> {
        self.conditional
            .get(&(location.clone(), ordinal))
            .copied()
            .map(roles_from_mask)
    }

    pub(super) const fn structural_membership(&self) -> bool {
        self.structural_membership
    }

    pub(super) fn contains_workflow_effect_receipt(&self, identity: &str) -> bool {
        self.workflow_effect_receipts.contains(identity)
    }

    pub(super) fn entry_count(&self) -> usize {
        self.native_contract.values().map(Vec::len).sum::<usize>()
            + self.native_whole.values().map(Vec::len).sum::<usize>()
            + self
                .native_field
                .values()
                .map(|index| index.len())
                .sum::<usize>()
            + self.collection_aspect.values().map(Vec::len).sum::<usize>()
            + self
                .collection_field
                .values()
                .map(|index| index.len())
                .sum::<usize>()
            + self.conditional.len()
            + self.workflow_effect_receipts.len()
    }

    pub(super) const fn mask_propagation_edges(&self) -> usize {
        self.mask_propagation_edges
    }
}

struct ImpactIndexCompilation {
    index: WorthQuerySemanticImpactIndex,
    conditional_declarations: Vec<(
        worth_query_installation::facade::WorthQueryConditionalNodeLocation,
        worth_query_installation::facade::WorthQueryPortableConditionalNodeDeclaration,
    )>,
    stage_predecessors: HashMap<String, Vec<String>>,
    stage_masks: HashMap<String, u16>,
}

impl ImpactIndexCompilation {
    fn new() -> Self {
        Self {
            index: WorthQuerySemanticImpactIndex {
                native_contract: HashMap::new(),
                native_whole: HashMap::new(),
                native_field: HashMap::new(),
                collection_aspect: HashMap::new(),
                collection_field: HashMap::new(),
                conditional: HashMap::new(),
                structural_membership: false,
                window_on_ordering: false,
                workflow_effect_receipts: HashSet::new(),
                mask_propagation_edges: 0,
            },
            conditional_declarations: Vec::new(),
            stage_predecessors: HashMap::new(),
            stage_masks: HashMap::new(),
        }
    }

    fn index_dependency(
        &mut self,
        dependency_ordinal: usize,
        dependency: &WorthQueryCompiledSemanticAspectDependency,
    ) {
        let edge = IndexedRoleEdge {
            dependency_ordinal,
            role: dependency.role(),
        };
        match &dependency.source {
            Source::NativeProjection(projection) => {
                self.index_native_projection(projection, edge)
            }
            Source::CollectionField(field) => self.index_collection_field(field, edge),
            Source::CollectionWindowPolicy(
                worth_query_installation::facade::WorthQueryOperationWindowPolicy::ContinuationBounded,
            ) => self.index.window_on_ordering = true,
            Source::ConditionalNodeContract(declaration) => {
                self.retain_conditional_declaration(dependency, declaration)
            }
            Source::WorkflowStage { predecessors } => {
                self.retain_stage_predecessors(dependency, predecessors)
            }
            Source::RealizedWorkflowEffect(evidence) => {
                self.index
                    .workflow_effect_receipts
                    .insert(evidence.receipt_identity().to_owned());
            }
            _ => {}
        }
        if !matches!(&dependency.source, Source::WorkflowStage { .. })
            && let Some(stage_identity) = dependency_stage_identity(&dependency.locus)
        {
            *self
                .stage_masks
                .entry(stage_identity.to_owned())
                .or_default() |= role_bit(dependency.role());
        }
    }

    fn index_native_projection(
        &mut self,
        projection: &worth_query_installation::facade::WorthQueryOperationNativeProjectionContract,
        edge: IndexedRoleEdge,
    ) {
        let contract = projection.contract();
        let key = (
            contract.key().clone(),
            contract.identity(),
            contract.revision(),
        );
        self.index
            .native_contract
            .entry(key.clone())
            .or_default()
            .push(edge.role);
        if projection.mask().is_whole_aspect() {
            self.index
                .native_whole
                .entry(key)
                .or_default()
                .push(edge.role);
        } else {
            for path in projection.mask().paths() {
                self.index
                    .native_field
                    .entry((key.0.clone(), key.1, key.2))
                    .or_default()
                    .insert(path, edge);
            }
        }
    }

    fn index_collection_field(
        &mut self,
        field: &worth_query_installation::facade::WorthQueryOperationCollectionField,
        edge: IndexedRoleEdge,
    ) {
        self.index
            .collection_aspect
            .entry(field.aspect_key().clone())
            .or_default()
            .push(edge.role);
        self.index
            .collection_field
            .entry(field.aspect_key().clone())
            .or_default()
            .insert(field.field_path(), edge);
        self.index.structural_membership |=
            edge.role == WorthQuerySemanticDependencyRole::SelectionOrMembership;
    }

    fn retain_conditional_declaration(
        &mut self,
        dependency: &WorthQueryCompiledSemanticAspectDependency,
        declaration: &worth_query_installation::facade::WorthQueryPortableConditionalNodeDeclaration,
    ) {
        if let super::dependency_locus::WorthQuerySemanticAspectDependencyLocus::ConditionalNode {
            location,
        } = &dependency.locus
        {
            self.conditional_declarations
                .push((location.clone(), declaration.clone()));
        }
    }

    fn retain_stage_predecessors(
        &mut self,
        dependency: &WorthQueryCompiledSemanticAspectDependency,
        predecessors: &[String],
    ) {
        if let super::dependency_locus::WorthQuerySemanticAspectDependencyLocus::WorkflowStage {
            stage_identity,
        } = &dependency.locus
        {
            self.stage_predecessors
                .insert(stage_identity.clone(), predecessors.to_vec());
        }
    }

    fn finish(mut self) -> WorthQuerySemanticImpactIndex {
        self.index.mask_propagation_edges =
            propagate_stage_consequences(&self.stage_predecessors, &mut self.stage_masks);
        let all_stage_mask = self
            .stage_masks
            .values()
            .copied()
            .fold(0, |left, right| left | right);
        for (location, declaration) in self.conditional_declarations {
            let stage_mask = location
                .stage_identity()
                .and_then(|stage| self.stage_masks.get(stage))
                .copied()
                .unwrap_or(all_stage_mask);
            let consequence_mask = conditional_output_mask(&declaration) | stage_mask;
            for dependency_ordinal in 0..declaration.dependencies().len() {
                self.index
                    .conditional
                    .insert((location.clone(), dependency_ordinal), consequence_mask);
            }
        }
        self.index
    }
}

fn extend(
    target: &mut Vec<WorthQuerySemanticDependencyRole>,
    source: Option<&Vec<WorthQuerySemanticDependencyRole>>,
    lookups: &mut usize,
) {
    *lookups += 1;
    if let Some(source) = source {
        target.extend_from_slice(source);
    }
}

fn extend_overlapping(
    target: &mut Vec<WorthQuerySemanticDependencyRole>,
    source: Option<
        &crate::canonical_field_path_overlap_index::WorthQueryCanonicalPathOverlapIndex<
            IndexedRoleEdge,
        >,
    >,
    path: &CanonicalFieldPath,
    lookups: &mut usize,
) {
    *lookups += 1;
    if let Some(source) = source {
        let (edges, work) = source.overlapping(path);
        *lookups += work.node_probes;
        target.extend(edges.into_iter().map(|edge| edge.role));
    }
}

#[cfg(test)]
mod tests;
