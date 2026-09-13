//! Product-local semantic output correspondence owned by Query publication.

use std::any::TypeId;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::Arc;

use worth_query_installation::facade::ApplicationSchemaBindingIdentity;

use super::{
    provider::WorthQueryPrimaryGraphCommittedApplication,
    WorthQueryApplicationOutputCorrespondence, WorthQueryApplicationOutputProjectionDenial,
};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct SemanticSource {
    runtime_authority: u64,
    schema: ApplicationSchemaBindingIdentity,
    scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
    output_binding: TypeId,
}

#[derive(Default)]
pub(crate) struct WorthQueryApplicationOutputLineage {
    by_source: HashMap<
        SemanticSource,
        HashMap<
            worth_runtime_world::facade::ProductBranchIncarnation,
            BTreeMap<u64, RecordedOutput>,
        >,
    >,
    origins: HashMap<worth_runtime_world::facade::ProductBranchIncarnation, ProductCoordinate>,
    live_occurrences: HashSet<worth_runtime_world::facade::ProductBranchIncarnation>,
}

struct RecordedOutput {
    correspondence: Arc<WorthQueryApplicationOutputCorrespondence>,
    source_identity: Option<[u8; 32]>,
}

#[derive(Clone, Copy)]
struct ProductCoordinate {
    occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
    generation: u64,
}

pub(super) struct WorthQueryPriorOutputBindingResolution {
    pub(super) correspondence: Option<Arc<WorthQueryApplicationOutputCorrespondence>>,
    pub(super) source_lookups: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryPriorOutputDenialKind {
    Unavailable,
    MissingRole,
    ActionMismatch,
    EntityMismatch,
    OutputUnavailable,
    UndeclaredDecisionTarget,
    WorkBudgetExceeded,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryPriorOutputDenial {
    kind: WorthQueryPriorOutputDenialKind,
    role: String,
}

impl WorthQueryApplicationOutputLineage {
    pub(super) fn source_posture_for_any_output_binding(
        &self,
        runtime_authority: u64,
        schema: &ApplicationSchemaBindingIdentity,
        scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
        occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
        generation: u64,
        output_bindings: &[TypeId],
        current_source_identity: [u8; 32],
    ) -> WorthQueryOutputSourcePosture {
        let mut retained_output = false;
        for output_binding in output_bindings {
            let source = SemanticSource {
                runtime_authority,
                schema: schema.clone(),
                scope,
                output_binding: *output_binding,
            };
            let Some(versions) = self.by_source.get(&source) else {
                continue;
            };
            let mut coordinate = ProductCoordinate {
                occurrence,
                generation,
            };
            loop {
                if versions.get(&coordinate.occurrence).is_some_and(|history| {
                    history
                        .range(..=coordinate.generation)
                        .next_back()
                        .is_some()
                }) {
                    let recorded = versions
                        .get(&coordinate.occurrence)
                        .and_then(|history| history.range(..=coordinate.generation).next_back())
                        .map(|(_, recorded)| recorded)
                        .expect("retained output was just found");
                    if recorded.source_identity == Some(current_source_identity) {
                        return WorthQueryOutputSourcePosture::Exact(*output_binding);
                    }
                    retained_output = true;
                }
                let Some(parent) = self.origins.get(&coordinate.occurrence).copied() else {
                    break;
                };
                coordinate = parent;
            }
        }
        if retained_output {
            WorthQueryOutputSourcePosture::Drifted
        } else {
            WorthQueryOutputSourcePosture::Absent
        }
    }

    pub(crate) fn register_fork(
        &mut self,
        source: &worth_runtime_world::facade::ProductBranchObservation,
        destination: &worth_runtime_world::facade::ProductBranchObservation,
    ) {
        let source = ProductCoordinate {
            occurrence: source.lifecycle_incarnation(),
            generation: source.reference_generation().get(),
        };
        let destination = destination.lifecycle_incarnation();
        self.live_occurrences.insert(source.occurrence);
        self.live_occurrences.insert(destination);
        assert!(
            self.origins.insert(destination, source).is_none(),
            "one product occurrence may be registered as a fork once"
        );
    }

    pub(super) fn record(&mut self, application: &WorthQueryPrimaryGraphCommittedApplication) {
        let evidence = application.commit_evidence();
        let correspondence = evidence.output_correspondence();
        let scope = evidence.operation_scope();
        let head = application.product_publication().new_product_head();
        let coordinate = ProductCoordinate {
            occurrence: head.lifecycle_incarnation(),
            generation: head.reference_generation().get(),
        };
        self.live_occurrences.insert(coordinate.occurrence);
        let Some(output_binding) = correspondence.binding_type() else {
            return;
        };
        let source = SemanticSource {
            runtime_authority: scope.runtime_authority(),
            schema: scope.binding_identity().clone(),
            scope: scope.scope(),
            output_binding,
        };
        let replaced = self
            .by_source
            .entry(source)
            .or_default()
            .entry(head.lifecycle_incarnation())
            .or_default()
            .insert(
                head.reference_generation().get(),
                RecordedOutput {
                    correspondence: evidence.retain_output_correspondence(),
                    source_identity: evidence.idempotency().source_identity(),
                },
            );
        assert!(
            replaced.is_none(),
            "one product generation may publish one output binding once"
        );
    }

    pub(super) fn resolve_binding<Binding: 'static>(
        &self,
        scope: &crate::domain_computation::authorization::WorthQueryOperationScopeBinding,
        occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
        generation: u64,
        maximum_source_lookups: usize,
    ) -> Result<WorthQueryPriorOutputBindingResolution, ()> {
        let source = SemanticSource {
            runtime_authority: scope.runtime_authority(),
            schema: scope.binding_identity().clone(),
            scope: scope.scope(),
            output_binding: TypeId::of::<Binding>(),
        };
        let Some(versions) = self.by_source.get(&source) else {
            return Ok(WorthQueryPriorOutputBindingResolution {
                correspondence: None,
                source_lookups: 1,
            });
        };
        let mut coordinate = ProductCoordinate {
            occurrence,
            generation,
        };
        let mut source_lookups = 0;
        loop {
            source_lookups += 1;
            if source_lookups > maximum_source_lookups {
                return Err(());
            }
            if let Some(correspondence) = versions
                .get(&coordinate.occurrence)
                .and_then(|history| history.range(..=coordinate.generation).next_back())
                .map(|(_, recorded)| recorded.correspondence.clone())
            {
                return Ok(WorthQueryPriorOutputBindingResolution {
                    correspondence: Some(correspondence),
                    source_lookups,
                });
            }
            let Some(parent) = self.origins.get(&coordinate.occurrence).copied() else {
                return Ok(WorthQueryPriorOutputBindingResolution {
                    correspondence: None,
                    source_lookups,
                });
            };
            coordinate = parent;
        }
    }

    pub(super) fn release_occurrence(
        &mut self,
        occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
    ) {
        self.live_occurrences.remove(&occurrence);
        let mut retained = self.live_occurrences.clone();
        let mut frontier = retained.iter().copied().collect::<Vec<_>>();
        while let Some(child) = frontier.pop() {
            if let Some(parent) = self
                .origins
                .get(&child)
                .map(|coordinate| coordinate.occurrence)
            {
                if retained.insert(parent) {
                    frontier.push(parent);
                }
            }
        }
        self.by_source.retain(|_, versions| {
            versions.retain(|indexed, _| retained.contains(indexed));
            !versions.is_empty()
        });
        self.origins.retain(|child, _| retained.contains(child));
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum WorthQueryOutputSourcePosture {
    Absent,
    Exact(TypeId),
    Drifted,
}

impl WorthQueryPriorOutputDenial {
    pub const fn kind(&self) -> WorthQueryPriorOutputDenialKind {
        self.kind
    }

    pub fn role(&self) -> &str {
        &self.role
    }

    pub(super) fn new(kind: WorthQueryPriorOutputDenialKind, role: impl Into<String>) -> Self {
        Self {
            kind,
            role: role.into(),
        }
    }

    pub(super) fn projection(
        role: &str,
        denial: WorthQueryApplicationOutputProjectionDenial,
    ) -> Self {
        let kind = match denial {
            WorthQueryApplicationOutputProjectionDenial::MissingRole => {
                WorthQueryPriorOutputDenialKind::MissingRole
            }
            WorthQueryApplicationOutputProjectionDenial::ForeignBinding => {
                WorthQueryPriorOutputDenialKind::Unavailable
            }
            WorthQueryApplicationOutputProjectionDenial::ActionMismatch => {
                WorthQueryPriorOutputDenialKind::ActionMismatch
            }
            WorthQueryApplicationOutputProjectionDenial::EntityMismatch => {
                WorthQueryPriorOutputDenialKind::EntityMismatch
            }
        };
        Self::new(kind, role)
    }
}

impl std::fmt::Display for WorthQueryPriorOutputDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "prior output denied: {:?} ({})",
            self.kind, self.role
        )
    }
}

impl std::error::Error for WorthQueryPriorOutputDenial {}
