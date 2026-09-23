//! Product-local semantic output correspondence owned by Query publication.

mod current_output;
mod qualification;
mod record;
mod retention;
#[cfg(test)]
mod tests;

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
    source_partition_identity: Option<[u8; 32]>,
    producer_dependency_identity: Option<[u8; 32]>,
    idempotency_key_identity: [u8; 32],
    observed_source_facts: Arc<[super::application_attempt::WorthQueryApplicationObservedFact]>,
}

pub(super) struct WorthQueryExactRecordedOutput {
    pub(super) correspondence: Arc<WorthQueryApplicationOutputCorrespondence>,
    pub(super) source_identity: [u8; 32],
    pub(super) source_partition_identity: [u8; 32],
    pub(super) producer_dependency_identity: Option<[u8; 32]>,
    pub(super) idempotency_key_identity: [u8; 32],
    pub(super) runtime_authority: u64,
    pub(super) schema: ApplicationSchemaBindingIdentity,
    pub(super) scope:
        crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
    pub(super) observed_source_facts:
        Arc<[super::application_attempt::WorthQueryApplicationObservedFact]>,
}

#[derive(Clone, Copy)]
pub(super) struct WorthQueryProducerLineageHead {
    pub(super) occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
    pub(super) dependency_identity: Option<[u8; 32]>,
    pub(super) idempotency_key_identity: [u8; 32],
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
        source_partition_identity: [u8; 32],
        maximum_source_lookups: usize,
    ) -> Result<WorthQueryPriorOutputBindingResolution, ()> {
        let source = SemanticSource {
            runtime_authority: scope.runtime_authority(),
            schema: scope.binding_identity().clone(),
            scope: scope.scope(),
            output_binding: TypeId::of::<Binding>(),
        };
        let Some(versions) = self.by_source.get(&source) else {
            if maximum_source_lookups == 0 {
                return Err(());
            }
            return Ok(WorthQueryPriorOutputBindingResolution {
                correspondence: None,
                source_lookups: 1,
            });
        };
        let mut coordinate = ProductCoordinate {
            occurrence,
            generation,
        };
        let mut source_lookups = 0_usize;
        loop {
            let (recorded, work) = match versions.get(&coordinate.occurrence) {
                Some(history) => latest_output_in_partition_budgeted(
                    history,
                    coordinate.generation,
                    source_partition_identity,
                    maximum_source_lookups.saturating_sub(source_lookups),
                )?,
                None => (None, 1),
            };
            source_lookups = source_lookups.checked_add(work).ok_or(())?;
            if source_lookups > maximum_source_lookups {
                return Err(());
            }
            if let Some((_, recorded)) = recorded {
                return Ok(WorthQueryPriorOutputBindingResolution {
                    correspondence: Some(Arc::clone(&recorded.correspondence)),
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

#[cfg(test)]
fn latest_output_in_partition(
    history: &BTreeMap<u64, RecordedOutput>,
    maximum_generation: u64,
    source_partition_identity: [u8; 32],
) -> Option<(&u64, &RecordedOutput)> {
    history
        .range(..=maximum_generation)
        .rev()
        .find(|(_, recorded)| recorded.source_partition_identity == Some(source_partition_identity))
}

fn latest_output_in_partition_budgeted(
    history: &BTreeMap<u64, RecordedOutput>,
    maximum_generation: u64,
    source_partition_identity: [u8; 32],
    maximum_work: usize,
) -> Result<(Option<(&u64, &RecordedOutput)>, usize), ()> {
    let mut work = 0_usize;
    for entry in history.range(..=maximum_generation).rev() {
        work = work.checked_add(1).ok_or(())?;
        if work > maximum_work {
            return Err(());
        }
        if entry.1.source_partition_identity == Some(source_partition_identity) {
            return Ok((Some(entry), work));
        }
    }
    // An empty generation range still performs one source-coordinate lookup.
    if work == 0 {
        if maximum_work == 0 {
            return Err(());
        }
        work = 1;
    }
    Ok((None, work))
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
