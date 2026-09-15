//! Product-local semantic output correspondence owned by Query publication.

mod current_output;
mod retention;

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
    output_families: HashMap<String, Vec<TypeId>>,
}

struct RecordedOutput {
    correspondence: Arc<WorthQueryApplicationOutputCorrespondence>,
    source_identity: Option<[u8; 32]>,
    observed_source_facts: Arc<[super::application_attempt::WorthQueryApplicationObservedFact]>,
}

pub(super) struct WorthQueryExactRecordedOutput {
    pub(super) correspondence: Arc<WorthQueryApplicationOutputCorrespondence>,
    pub(super) source_identity: [u8; 32],
    pub(super) runtime_authority: u64,
    pub(super) schema: ApplicationSchemaBindingIdentity,
    pub(super) scope:
        crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
    pub(super) observed_source_facts:
        Arc<[super::application_attempt::WorthQueryApplicationObservedFact]>,
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

pub(super) struct WorthQueryCurrentOutputCandidate {
    pub(super) correspondence: Arc<WorthQueryApplicationOutputCorrespondence>,
    pub(super) observed_source_facts:
        Arc<[super::application_attempt::WorthQueryApplicationObservedFact]>,
}

pub(super) struct WorthQueryCurrentOutputFamilyResolution {
    pub(super) family_installed: bool,
    pub(super) candidates: Vec<WorthQueryCurrentOutputCandidate>,
    pub(super) source_lookups: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryPriorOutputDenialKind {
    Unavailable,
    UndeclaredFamily,
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
    pub(super) fn exact_output<Binding: 'static>(
        &self,
        occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
        generation: u64,
    ) -> Option<WorthQueryExactRecordedOutput> {
        let mut matching = self.by_source.iter().filter_map(|(source, versions)| {
            (source.output_binding == TypeId::of::<Binding>()).then(|| {
                versions
                    .get(&occurrence)?
                    .get(&generation)
                    .map(|recorded| (source, recorded))
            })?
        });
        let (source, recorded) = matching.next()?;
        assert!(
            matching.next().is_none(),
            "one product generation has one output binding"
        );
        Some(WorthQueryExactRecordedOutput {
            correspondence: Arc::clone(&recorded.correspondence),
            source_identity: recorded.source_identity?,
            runtime_authority: source.runtime_authority,
            schema: source.schema.clone(),
            scope: source.scope,
            observed_source_facts: Arc::clone(&recorded.observed_source_facts),
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn record_restoration(
        &mut self,
        output_binding: TypeId,
        runtime_authority: u64,
        schema: ApplicationSchemaBindingIdentity,
        scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
        observation: &worth_runtime_world::facade::ProductBranchObservation,
        correspondence: Arc<WorthQueryApplicationOutputCorrespondence>,
        source_identity: [u8; 32],
        observed_source_facts: Arc<[super::application_attempt::WorthQueryApplicationObservedFact]>,
    ) {
        let source = SemanticSource {
            runtime_authority,
            schema,
            scope,
            output_binding,
        };
        let replaced = self
            .by_source
            .entry(source)
            .or_default()
            .entry(observation.lifecycle_incarnation())
            .or_default()
            .insert(
                observation.reference_generation().get(),
                RecordedOutput {
                    correspondence,
                    source_identity: Some(source_identity),
                    observed_source_facts,
                },
            );
        assert!(
            replaced.is_none(),
            "one product generation may restore one output binding once"
        );
        self.live_occurrences
            .insert(observation.lifecycle_incarnation());
    }

    pub(super) fn install_output_families(&mut self, families: BTreeMap<String, Vec<TypeId>>) {
        assert!(self.output_families.is_empty());
        self.output_families.extend(families);
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
                    observed_source_facts: evidence.retain_observed_source_facts(),
                },
            );
        assert!(
            replaced.is_none(),
            "one product generation may publish one output binding once"
        );
    }

    pub(super) fn resolve_current_family(
        &self,
        runtime_authority: u64,
        schema: &ApplicationSchemaBindingIdentity,
        scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
        family: &str,
        occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
        generation: u64,
        maximum_source_lookups: usize,
    ) -> Result<WorthQueryCurrentOutputFamilyResolution, ()> {
        let Some(bindings) = self.output_families.get(family) else {
            return Ok(WorthQueryCurrentOutputFamilyResolution {
                family_installed: false,
                candidates: Vec::new(),
                source_lookups: 1,
            });
        };
        let mut candidates = Vec::new();
        let mut source_lookups = 0_usize;
        for output_binding in bindings {
            let source = SemanticSource {
                runtime_authority,
                schema: schema.clone(),
                scope,
                output_binding: *output_binding,
            };
            source_lookups = source_lookups.saturating_add(1);
            if source_lookups > maximum_source_lookups {
                return Err(());
            }
            let Some(versions) = self.by_source.get(&source) else {
                continue;
            };
            let mut coordinate = ProductCoordinate {
                occurrence,
                generation,
            };
            loop {
                source_lookups = source_lookups.saturating_add(1);
                if source_lookups > maximum_source_lookups {
                    return Err(());
                }
                if let Some(recorded) = versions
                    .get(&coordinate.occurrence)
                    .and_then(|history| history.range(..=coordinate.generation).next_back())
                    .map(|(_, recorded)| recorded)
                {
                    candidates.push(WorthQueryCurrentOutputCandidate {
                        correspondence: Arc::clone(&recorded.correspondence),
                        observed_source_facts: Arc::clone(&recorded.observed_source_facts),
                    });
                    break;
                }
                let Some(parent) = self.origins.get(&coordinate.occurrence).copied() else {
                    break;
                };
                coordinate = parent;
            }
        }
        Ok(WorthQueryCurrentOutputFamilyResolution {
            family_installed: true,
            candidates,
            source_lookups,
        })
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
