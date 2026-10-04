//! Candidate heads for every source partition in a producer-qualified output family.
use super::{
    ProductCoordinate, SemanticSource, WorthQueryApplicationOutputLineage,
    WorthQueryCurrentOutputCandidate, WorthQueryCurrentOutputFamilyResolution,
};
use std::{collections::BTreeSet, sync::Arc};
use worth_query_installation::facade::ApplicationSchemaBindingIdentity;

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
            return (maximum_source_lookups > 0)
                .then_some(WorthQueryCurrentOutputFamilyResolution {
                    family_installed: false,
                    candidates: Vec::new(),
                    source_lookups: 1,
                })
                .ok_or(());
        };
        let mut candidates = Vec::new();
        let mut source_lookups = 0usize;
        for (output_binding, output_role) in bindings {
            source_lookups = source_lookups.checked_add(1).ok_or(())?;
            if source_lookups > maximum_source_lookups {
                return Err(());
            }
            let source = SemanticSource {
                runtime_authority,
                schema: schema.clone(),
                scope,
                output_binding: *output_binding,
            };
            if !self.by_source.contains_key(&source) {
                continue;
            }
            let mut coordinate = ProductCoordinate {
                occurrence,
                generation,
            };
            let mut seen = BTreeSet::new();
            loop {
                let (heads, work) = self.family_partition_heads_budgeted(
                    &source,
                    coordinate,
                    maximum_source_lookups.saturating_sub(source_lookups),
                )?;
                source_lookups = source_lookups.checked_add(work).ok_or(())?;
                for (partition, recorded) in heads {
                    if !seen.insert(partition) {
                        continue;
                    }
                    // A newer descriptive head must not revive an older fact-bearing output.
                    if let Some(facts) = &recorded.observed_source_facts {
                        candidates.push(WorthQueryCurrentOutputCandidate {
                            correspondence: Arc::clone(&recorded.correspondence),
                            output_role: output_role.clone(),
                            observed_source_facts: Arc::clone(facts),
                        });
                    }
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
}
