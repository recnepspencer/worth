//! Candidate heads for every source partition in a producer-qualified output family.
use super::{
    ProductCoordinate, SemanticSource, WorthQueryApplicationOutputLineage,
    WorthQueryCurrentOutputCandidate, WorthQueryCurrentOutputFamilyResolution,
};
use std::{collections::BTreeSet, sync::Arc};
use worth_query_installation::facade::ApplicationSchemaBindingIdentity;
mod checkpoint_priors;
mod publication_heads;
pub(in crate::domain_computation::primary_graph) use checkpoint_priors::NativePriorCheckpointOutput;
pub(in crate::domain_computation::primary_graph) use checkpoint_priors::{
    CheckpointPriorSelectionDenial, CheckpointPriorStructure,
};
use publication_heads::PublicationHeads;

impl WorthQueryApplicationOutputLineage {
    pub(in crate::domain_computation::primary_graph) fn checkpoint_family_role(
        &self,
        binding: Option<std::any::TypeId>,
        admission: &mut super::InvalidationEditAdmission,
    ) -> Result<Option<(&str, &str)>, CheckpointPriorSelectionDenial> {
        let Some(binding) = binding else {
            return Ok(None);
        };
        let mut found = None;
        for (family, bindings) in &self.output_families {
            admission.charge_external_work(1).map_err(|stop| {
                CheckpointPriorSelectionDenial::Admission {
                    phase: "accepted family inventory visit",
                    stop,
                }
            })?;
            for (candidate, role) in bindings {
                admission.charge_external_work(1).map_err(|stop| {
                    CheckpointPriorSelectionDenial::Admission {
                        phase: "accepted binding inventory visit",
                        stop,
                    }
                })?;
                if *candidate != binding {
                    continue;
                }
                if found.is_some() {
                    return Err(CheckpointPriorSelectionDenial::Structural(
                        CheckpointPriorStructure::DuplicateInstalledBinding,
                    ));
                }
                found = Some((family.as_str(), role.as_str()));
            }
        }
        Ok(found)
    }

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
                output_binding: self
                    .binding_identity(*output_binding)
                    .expect("output family binding is installed"),
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
                let (heads, work) = self
                    .family_partition_heads_budgeted(
                        &source,
                        coordinate,
                        maximum_selection_work.saturating_sub(selection_work),
                    )
                    .map_err(|_| ())?;
                selection_work = selection_work.checked_add(work).ok_or(())?;
                for (partition, publication) in heads {
                    if !seen.insert(partition) {
                        continue;
                    }
                    let work = publications.admission_work();
                    selection_work = selection_work.checked_add(work).ok_or(())?;
                    if selection_work > maximum_selection_work {
                        return Err(());
                    }
                    publications.insert(
                        partition,
                        output_role,
                        *output_binding,
                        ancestry_depth,
                        publication,
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
        // Descriptive and retirement heads participate in publication selection,
        // so an older binding cannot revive a fact-bearing predecessor.
        let candidates = heads
            .into_iter()
            .filter_map(|(output_role, recorded)| {
                recorded
                    .observed_source_facts()
                    .map(|facts| WorthQueryCurrentOutputCandidate {
                        consumed_outputs: Arc::clone(&recorded.consumed_outputs),
                        verification_requirement: recorded.verification_requirement(),
                        settlement_identity: Arc::clone(&recorded.settlement_identity),
                        correspondence: Arc::clone(&recorded.correspondence),
                        output_role: output_role.to_owned(),
                        observed_source_facts: facts,
                        native_output_witness: recorded
                            .performed_origin
                            .as_ref()
                            .and_then(|origin| origin.get())
                            .unwrap_or(recorded)
                            .native_output_witness_cell()
                            .map(Arc::clone),
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
