//! Candidate heads for every source partition in a producer-qualified output family.
use super::{
    ProductCoordinate, SemanticSource, WorthQueryApplicationOutputLineage,
    WorthQueryCurrentOutputCandidate, WorthQueryCurrentOutputFamilyResolution,
};
use std::{collections::BTreeSet, sync::Arc};
use worth_query_installation::facade::ApplicationSchemaBindingIdentity;
mod publication_heads;
use publication_heads::PublicationHeads;

impl WorthQueryApplicationOutputLineage {
    pub(in crate::domain_computation::primary_graph) fn resolve_current_family(
        &self,
        runtime_authority: u64,
        schema: &ApplicationSchemaBindingIdentity,
        scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
        family: &str,
        occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
        generation: u64,
        maximum_selection_work: usize,
    ) -> Result<WorthQueryCurrentOutputFamilyResolution, ()> {
        let Some(bindings) = self.output_families.get(family) else {
            return (maximum_selection_work > 0)
                .then_some(WorthQueryCurrentOutputFamilyResolution {
                    family_installed: false,
                    ambiguous_publication: false,
                    candidates: Vec::new(),
                    selection_work: 1,
                })
                .ok_or(());
        };
        let mut publications = PublicationHeads::default();
        let mut selection_work = 0usize;
        for (output_binding, output_role) in bindings {
            selection_work = selection_work.checked_add(1).ok_or(())?;
            if selection_work > maximum_selection_work {
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
            let mut ancestry_depth = 0usize;
            loop {
                let (heads, work) = self.family_partition_heads_budgeted(
                    &source,
                    coordinate,
                    maximum_selection_work.saturating_sub(selection_work),
                )?;
                selection_work = selection_work.checked_add(work).ok_or(())?;
                for (partition, head) in heads {
                    if !seen.insert(partition) {
                        continue;
                    }
                    selection_work = selection_work
                        .checked_add(publications.admission_work())
                        .ok_or(())?;
                    if selection_work > maximum_selection_work {
                        return Err(());
                    }
                    publications.insert(
                        partition,
                        output_role,
                        *output_binding,
                        ancestry_depth,
                        head,
                    );
                }
                let Some(parent) = self.origins.get(&coordinate.occurrence).copied() else {
                    break;
                };
                coordinate = parent;
                ancestry_depth = ancestry_depth.checked_add(1).ok_or(())?;
            }
        }
        let (heads, ambiguous_publication) = publications.finish();
        // Descriptive and retirement publications suppress predecessors before
        // fact filtering, so another binding cannot revive obsolete evidence.
        let candidates = heads
            .into_iter()
            .filter_map(|(output_role, recorded)| {
                recorded.observed_source_facts.as_ref().map(|facts| {
                    WorthQueryCurrentOutputCandidate {
                        correspondence: Arc::clone(&recorded.correspondence),
                        output_role: output_role.to_owned(),
                        observed_source_facts: Arc::clone(facts),
                    }
                })
            })
            .collect();
        Ok(WorthQueryCurrentOutputFamilyResolution {
            family_installed: true,
            ambiguous_publication,
            candidates,
            selection_work,
        })
    }
}
