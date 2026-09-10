use std::collections::{BTreeMap, BTreeSet};

use worth_foundational::facade::{AspectKey, CanonicalFieldPath};

use super::super::compiled::{
    WorthQueryCompiledSemanticAspectDependencyClosure, WorthQuerySemanticAspectDependencyView,
    WorthQuerySemanticDependencyClosureEvidence, WorthQuerySemanticDependencyRole,
};
use super::{WorthQueryImpactClass, WorthQueryImpactCounters};

mod routing_selector;

pub(crate) use routing_selector::WorthQueryInstalledLiveRoutingSelector;

#[derive(Clone)]
pub(crate) struct WorthQueryInstalledLiveImpactClassifier {
    affinity:
        crate::domain_installation::operation_authority_chain::WorthQueryOperationAuthorityBasis,
    closure_evidence: WorthQuerySemanticDependencyClosureEvidence,
    aspect_roles: BTreeMap<AspectKey, Vec<WorthQuerySemanticDependencyRole>>,
    whole_roles: BTreeMap<AspectKey, Vec<WorthQuerySemanticDependencyRole>>,
    field_roles: BTreeMap<
        AspectKey,
        crate::canonical_field_path_overlap_index::WorthQueryCanonicalPathOverlapIndex<
            WorthQuerySemanticDependencyRole,
        >,
    >,
    field_routes: BTreeSet<(AspectKey, CanonicalFieldPath)>,
    ambiguous_native_aspects: BTreeSet<AspectKey>,
    conditional_aspects: BTreeSet<AspectKey>,
    conditional_broad_locality: bool,
    continuation_window_on_ordering_or_grouping: bool,
    structural_roles: Vec<WorthQuerySemanticDependencyRole>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WorthQueryPreclassifiedInstalledLiveImpact {
    affinity:
        crate::domain_installation::operation_authority_chain::WorthQueryOperationAuthorityBasis,
    closure_evidence: WorthQuerySemanticDependencyClosureEvidence,
    mutation: crate::memory_workspace::WorthQueryMutationDelta,
    class: WorthQueryImpactClass,
    roles: Vec<WorthQuerySemanticDependencyRole>,
    counters: WorthQueryImpactCounters,
}

impl WorthQueryInstalledLiveImpactClassifier {
    pub(crate) fn from_closure(
        closure: &WorthQueryCompiledSemanticAspectDependencyClosure,
    ) -> Self {
        let mut classifier = Self {
            affinity: closure.affinity.clone(),
            closure_evidence: closure.closure_evidence(),
            aspect_roles: BTreeMap::new(),
            whole_roles: BTreeMap::new(),
            field_roles: BTreeMap::new(),
            field_routes: BTreeSet::new(),
            ambiguous_native_aspects: BTreeSet::new(),
            conditional_aspects: BTreeSet::new(),
            conditional_broad_locality: false,
            continuation_window_on_ordering_or_grouping: false,
            structural_roles: Vec::new(),
        };
        let mut native_contracts = BTreeMap::<
            AspectKey,
            BTreeSet<(
                worth_foundational::facade::AspectIdentity,
                worth_foundational::facade::AspectContractRevision,
            )>,
        >::new();
        for dependency in closure.dependencies() {
            let role = dependency.role();
            match dependency.source() {
                WorthQuerySemanticAspectDependencyView::NativeProjection(projection) => {
                    let aspect = projection.contract().key().clone();
                    native_contracts.entry(aspect.clone()).or_default().insert((
                        projection.contract().identity(),
                        projection.contract().revision(),
                    ));
                    push_role(&mut classifier.aspect_roles, aspect.clone(), role);
                    if projection.mask().is_whole_aspect() {
                        push_role(&mut classifier.whole_roles, aspect, role);
                    } else {
                        for path in projection.mask().paths() {
                            classifier
                                .field_roles
                                .entry(aspect.clone())
                                .or_default()
                                .insert(path, role);
                            classifier
                                .field_routes
                                .insert((aspect.clone(), path.clone()));
                        }
                    }
                }
                WorthQuerySemanticAspectDependencyView::CollectionField(field) => {
                    push_role(
                        &mut classifier.aspect_roles,
                        field.aspect_key().clone(),
                        role,
                    );
                    classifier
                        .field_roles
                        .entry(field.aspect_key().clone())
                        .or_default()
                        .insert(field.field_path(), role);
                    classifier.field_routes.insert((
                        field.aspect_key().clone(),
                        field.field_path().clone(),
                    ));
                }
                WorthQuerySemanticAspectDependencyView::CollectionWindowPolicy(
                    worth_query_installation::facade::WorthQueryOperationWindowPolicy::ContinuationBounded,
                ) => {
                    classifier.continuation_window_on_ordering_or_grouping = true;
                }
                WorthQuerySemanticAspectDependencyView::ConditionalTruth(conditional) => {
                    classifier
                        .conditional_aspects
                        .insert(conditional.contract().key().clone());
                    classifier.conditional_broad_locality |= !matches!(
                        conditional.locality(),
                        worth_query_installation::facade::WorthQuerySemanticLocality::SourceRecord
                    );
                }
                _ => {}
            }
            if role == WorthQuerySemanticDependencyRole::SelectionOrMembership
                && !classifier.structural_roles.contains(&role)
            {
                classifier.structural_roles.push(role);
            }
        }
        classifier.ambiguous_native_aspects = native_contracts
            .into_iter()
            .filter_map(|(aspect, contracts)| (contracts.len() > 1).then_some(aspect))
            .collect();
        classifier
    }

    pub(crate) fn classify(
        &self,
        mutation: &crate::memory_workspace::WorthQueryMutationDelta,
    ) -> WorthQueryPreclassifiedInstalledLiveImpact {
        let mut roles = BTreeSet::new();
        let mut lookups = 0;
        let structural_floor = match mutation.kind() {
            crate::memory_workspace::WorthQueryMutationKind::Deleted
                if self
                    .structural_roles
                    .contains(&WorthQuerySemanticDependencyRole::SelectionOrMembership) =>
            {
                WorthQueryImpactClass::MembershipSplice
            }
            crate::memory_workspace::WorthQueryMutationKind::Deleted => {
                WorthQueryImpactClass::Retirement
            }
            _ => WorthQueryImpactClass::UnaffectedOrSuppressed,
        };
        let ambiguous_native = mutation.admitted_touched_aspects().iter().any(|touch| {
            self.ambiguous_native_aspects
                .contains(touch.native_aspect_key())
        });
        let conditional_without_signal = mutation_requires_signal_authority(
            &self.conditional_aspects,
            self.conditional_broad_locality,
            mutation,
        );
        if matches!(
            mutation.kind(),
            crate::memory_workspace::WorthQueryMutationKind::Created
                | crate::memory_workspace::WorthQueryMutationKind::Deleted
        ) {
            roles.extend(self.structural_roles.iter().copied());
            lookups += 1;
        }
        for touch in mutation.admitted_touched_aspects() {
            if let Some(path) = touch.native_field_path() {
                extend_roles(
                    &mut roles,
                    self.whole_roles.get(touch.native_aspect_key()),
                    &mut lookups,
                );
                extend_overlapping_roles(
                    &mut roles,
                    self.field_roles.get(touch.native_aspect_key()),
                    path,
                    &mut lookups,
                );
            } else {
                extend_roles(
                    &mut roles,
                    self.aspect_roles.get(touch.native_aspect_key()),
                    &mut lookups,
                );
            }
        }
        if self.continuation_window_on_ordering_or_grouping
            && roles.iter().any(|role| {
                matches!(
                    role,
                    WorthQuerySemanticDependencyRole::Ordering
                        | WorthQuerySemanticDependencyRole::Grouping
                )
            })
        {
            roles.insert(WorthQuerySemanticDependencyRole::WindowBoundary);
        }
        let roles = roles.into_iter().collect::<Vec<_>>();
        let class = if ambiguous_native || conditional_without_signal {
            WorthQueryImpactClass::UnsupportedEscalation
        } else {
            let role_class = roles.iter().copied().fold(
                WorthQueryImpactClass::UnaffectedOrSuppressed,
                |class, role| widen(class, role_class(role)),
            );
            widen(role_class, structural_floor)
        };
        WorthQueryPreclassifiedInstalledLiveImpact {
            affinity: self.affinity.clone(),
            closure_evidence: self.closure_evidence,
            mutation: mutation.clone(),
            class,
            roles: roles.clone(),
            counters: WorthQueryImpactCounters {
                owner_changes_inspected: 1,
                index_lookups: lookups,
                affected_edges: roles.len(),
                ..Default::default()
            },
        }
    }
}

impl WorthQueryPreclassifiedInstalledLiveImpact {
    pub(crate) fn readmit(
        &self,
        closure: &WorthQueryCompiledSemanticAspectDependencyClosure,
        mutation: &crate::memory_workspace::WorthQueryMutationDelta,
    ) -> bool {
        exact_affinity_match(&self.affinity, &closure.affinity)
            && self.closure_evidence == closure.closure_evidence()
            && &self.mutation == mutation
    }

    pub(crate) const fn class(&self) -> WorthQueryImpactClass {
        self.class
    }
    pub(crate) fn roles(&self) -> &[WorthQuerySemanticDependencyRole] {
        &self.roles
    }
    pub(crate) const fn counters(&self) -> WorthQueryImpactCounters {
        self.counters
    }
}

fn push_role<K: Ord>(
    index: &mut BTreeMap<K, Vec<WorthQuerySemanticDependencyRole>>,
    key: K,
    role: WorthQuerySemanticDependencyRole,
) {
    let roles = index.entry(key).or_default();
    if !roles.contains(&role) {
        roles.push(role);
    }
}

fn extend_roles(
    target: &mut BTreeSet<WorthQuerySemanticDependencyRole>,
    source: Option<&Vec<WorthQuerySemanticDependencyRole>>,
    lookups: &mut usize,
) {
    *lookups += 1;
    if let Some(source) = source {
        target.extend(source.iter().copied());
    }
}

fn extend_overlapping_roles(
    target: &mut BTreeSet<WorthQuerySemanticDependencyRole>,
    source: Option<
        &crate::canonical_field_path_overlap_index::WorthQueryCanonicalPathOverlapIndex<
            WorthQuerySemanticDependencyRole,
        >,
    >,
    path: &CanonicalFieldPath,
    lookups: &mut usize,
) {
    *lookups += 1;
    if let Some(source) = source {
        let (overlapping, work) = source.overlapping(path);
        *lookups += work.node_probes;
        target.extend(overlapping);
    }
}

fn role_class(role: WorthQuerySemanticDependencyRole) -> WorthQueryImpactClass {
    match role {
        WorthQuerySemanticDependencyRole::OperationalIdentity => WorthQueryImpactClass::Replacement,
        WorthQuerySemanticDependencyRole::SelectionOrMembership => {
            WorthQueryImpactClass::MembershipSplice
        }
        WorthQuerySemanticDependencyRole::Ordering | WorthQuerySemanticDependencyRole::Grouping => {
            WorthQueryImpactClass::ReorderOrRegroup
        }
        WorthQuerySemanticDependencyRole::ProjectedValue
        | WorthQuerySemanticDependencyRole::ConditionalEligibilityOrSemanticCleanliness => {
            WorthQueryImpactClass::ValuePatch
        }
        WorthQuerySemanticDependencyRole::WindowBoundary => WorthQueryImpactClass::WindowShift,
        WorthQuerySemanticDependencyRole::SupportAndLifecycle => {
            WorthQueryImpactClass::ExplicitRebind
        }
        WorthQuerySemanticDependencyRole::InstalledDomainInvariant => {
            WorthQueryImpactClass::Reexecute
        }
        WorthQuerySemanticDependencyRole::AdvisoryOnlyContext => {
            WorthQueryImpactClass::UnaffectedOrSuppressed
        }
    }
}

fn widen(left: WorthQueryImpactClass, right: WorthQueryImpactClass) -> WorthQueryImpactClass {
    if rank(left) >= rank(right) {
        left
    } else {
        right
    }
}

fn rank(class: WorthQueryImpactClass) -> u8 {
    match class {
        WorthQueryImpactClass::UnaffectedOrSuppressed => 0,
        WorthQueryImpactClass::ValuePatch => 1,
        WorthQueryImpactClass::MembershipSplice => 2,
        WorthQueryImpactClass::ReorderOrRegroup => 3,
        WorthQueryImpactClass::WindowShift => 4,
        WorthQueryImpactClass::Reexecute => 5,
        WorthQueryImpactClass::ExplicitRebind => 6,
        WorthQueryImpactClass::Replacement => 7,
        WorthQueryImpactClass::Retirement => 8,
        WorthQueryImpactClass::UnsupportedEscalation => 9,
    }
}

fn mutation_requires_signal_authority(
    conditional_aspects: &BTreeSet<AspectKey>,
    broad_locality: bool,
    mutation: &crate::memory_workspace::WorthQueryMutationDelta,
) -> bool {
    broad_locality
        || mutation
            .admitted_touched_aspects()
            .iter()
            .any(|touch| conditional_aspects.contains(touch.native_aspect_key()))
        || (!conditional_aspects.is_empty() && mutation.admitted_touched_aspects().is_empty())
}

fn exact_affinity_match(
    retained: &crate::domain_installation::operation_authority_chain::WorthQueryOperationAuthorityBasis,
    candidate: &crate::domain_installation::operation_authority_chain::WorthQueryOperationAuthorityBasis,
) -> bool {
    retained == candidate
}

#[cfg(test)]
#[path = "installed_live_tests.rs"]
mod tests;
