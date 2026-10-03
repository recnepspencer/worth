//! Bounded selection of retained output families and bindings.

use std::any::TypeId;
use std::collections::BTreeMap;
use std::sync::Arc;

use worth_query_installation::facade::ApplicationSchemaBindingIdentity;

use super::*;

impl WorthQueryApplicationOutputLineage {
    pub(in crate::domain_computation::primary_graph) fn resolve_current_family(
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
                let mut selected = None;
                if let Some(history) = versions.get(&coordinate.occurrence) {
                    for (_, records) in history.range(..=coordinate.generation).rev() {
                        source_lookups = source_lookups.saturating_add(1);
                        if source_lookups > maximum_source_lookups {
                            return Err(());
                        }
                        selected = records
                            .iter()
                            .rev()
                            .filter_map(|cell| cell.get())
                            .find(|recorded| recorded.observed_source_facts().is_some());
                        if selected.is_some() {
                            break;
                        }
                    }
                }
                if let Some(recorded) = selected {
                    source_lookups = source_lookups.checked_add(1).ok_or(())?;
                    if source_lookups > maximum_source_lookups {
                        return Err(());
                    }
                    candidates.push(WorthQueryCurrentOutputCandidate {
                        consumed_outputs: Arc::clone(&recorded.consumed_outputs),
                        verification_requirement: recorded.verification_requirement(),
                        settlement_identity: Arc::clone(&recorded.settlement_identity),
                        correspondence: Arc::clone(&recorded.correspondence),
                        observed_source_facts: recorded
                            .observed_source_facts()
                            .expect("a current-output candidate has source facts"),
                        native_output_witness: recorded
                            .performed_origin
                            .as_ref()
                            .and_then(|origin| origin.get())
                            .unwrap_or(recorded)
                            .native_output_witness_cell()
                            .map(Arc::clone),
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

    pub(in crate::domain_computation::primary_graph) fn resolve_binding<Binding: 'static>(
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
        if !self.by_source.contains_key(&source) {
            return (maximum_source_lookups > 0)
                .then_some(WorthQueryPriorOutputBindingResolution {
                    correspondence: None,
                    source_lookups: 1,
                })
                .ok_or(());
        }
        let mut coordinate = ProductCoordinate {
            occurrence,
            generation,
        };
        let mut source_lookups = 0_usize;
        loop {
            let (recorded, work) = self.latest_output_in_partition_budgeted(
                &source,
                coordinate,
                source_partition_identity,
                maximum_source_lookups.saturating_sub(source_lookups),
            )?;
            source_lookups = source_lookups.checked_add(work).ok_or(())?;
            if source_lookups > maximum_source_lookups {
                return Err(());
            }
            if let Some(recorded) = recorded {
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

pub(super) fn latest_output_matching(
    history: &BTreeMap<u64, RecordedGeneration>,
    maximum_generation: u64,
    mut matches: impl FnMut(&RecordedOutput) -> bool,
) -> Option<&RecordedOutput> {
    history
        .range(..=maximum_generation)
        .rev()
        .find_map(|(_, recorded)| {
            recorded
                .iter()
                .filter_map(|cell| cell.get())
                .find(|recorded| matches(recorded))
        })
}
