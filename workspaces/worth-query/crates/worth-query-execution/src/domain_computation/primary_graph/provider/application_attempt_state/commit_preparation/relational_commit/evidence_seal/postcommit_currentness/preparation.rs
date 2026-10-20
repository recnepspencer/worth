//! Pre-effect selection from genuine native candidate touches, with explicit full reconstruction.
use super::*;
use crate::domain_computation::primary_graph::{
    application_attempt::retained_decision_facts::{StorageControl, StoreDenial},
    output_lineage::invalidation::PostconditionCarry,
};
use worth_relational::facade::mvcc::PreparedRelationalCommitCandidate;
pub(super) enum Preparation {
    Full(FullSourceFactRebase),
    FullPayload {
        facts: FactPayload,
        moved: Arc<[usize]>,
    },
    Pending {
        facts: FactPayload,
        moved: Arc<[usize]>,
        carry: PostconditionCarry,
    },
    Selected(selected::SelectedRebase),
}
impl PreparedSourceFactRebase {
    pub(in crate::domain_computation::primary_graph) fn admit(
        facts: Vec<WorthQueryApplicationObservedFact>,
        moved: Arc<[usize]>,
        control: StorageControl<'_, '_>,
    ) -> Result<Self, PreparedRebaseDenial> {
        Ok(Self {
            preparation: Preparation::Full(FullSourceFactRebase::admit(facts, moved, control)?),
        })
    }
    pub(in crate::domain_computation::primary_graph) fn admit_output(
        facts: FactPayload,
        moved: Arc<[usize]>,
        carry: Option<PostconditionCarry>,
        control: StorageControl<'_, '_>,
    ) -> Result<Self, PreparedRebaseDenial> {
        control.check_live()?;
        match carry {
            Some(carry) if carry.prefix() <= facts.len() => Ok(Self {
                preparation: Preparation::Pending {
                    facts,
                    moved,
                    carry,
                },
            }),
            _ => Self::full_payload(facts, moved, control),
        }
    }
    fn full_payload(
        facts: FactPayload,
        moved: Arc<[usize]>,
        control: StorageControl<'_, '_>,
    ) -> Result<Self, PreparedRebaseDenial> {
        control.check_live()?;
        Ok(Self {
            preparation: Preparation::FullPayload { facts, moved },
        })
    }
    pub(in crate::domain_computation::primary_graph) fn prepare_candidate(
        self,
        candidate: &PreparedRelationalCommitCandidate,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        before: &worth_relational::facade::snapshots::SnapshotHandle,
        admission: &mut crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission,
        control: StorageControl<'_, '_>,
    ) -> Result<Self, PreparedRebaseDenial> {
        let preparation = match self.preparation {
            Preparation::FullPayload { facts, moved } => {
                return Ok(Self {
                    preparation: Preparation::Selected(selected::SelectedRebase::prepare_full(
                        facts, moved, control,
                    )?),
                })
            }
            preparation => preparation,
        };
        let Preparation::Pending {
            facts,
            moved,
            carry,
        } = preparation
        else {
            return Ok(Self { preparation });
        };
        control.check_live()?;
        if !carry
            .matches_publication_source(runtime, before, admission)
            .map_err(PreparedRebaseDenial::Admission)?
        {
            crate::domain_computation::primary_graph::fact_verification_observation::native_completion_decline("postcondition-publication-source");
            return Ok(Self {
                preparation: Preparation::Selected(selected::SelectedRebase::prepare_full(
                    facts, moved, control,
                )?),
            });
        }
        control.check_live()?;
        let selection = candidate
            .with_prepared_descriptive_touches(|touches| {
                carry.select(touches, facts.len(), &moved, control)
            })
            .ok_or(StoreDenial::ConflictingBody)??;
        let Some(selection) = selection else {
            return Ok(Self {
                preparation: Preparation::Selected(selected::SelectedRebase::prepare_full(
                    facts, moved, control,
                )?),
            });
        };
        Ok(Self {
            preparation: Preparation::Selected(selected::SelectedRebase::prepare(
                facts,
                moved,
                carry,
                selection.ordinals,
                selection.memory,
                selection.basis,
                admission,
                control,
            )?),
        })
    }
}
#[allow(clippy::too_many_arguments)]
pub(super) fn rebase(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    prepared: PreparedSourceFactRebase,
    retired: &BTreeSet<EntityId>,
    producer: bool,
    maximum: Option<usize>,
    admission: &mut crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission,
    control: StorageControl<'_, '_>,
) -> RebasedSourceFacts {
    match prepared.preparation {
        Preparation::Full(full) => rebase_walk::rebase(
            execution, runtime, snapshot, full, retired, producer, maximum, admission,
        ),
        Preparation::Selected(selected) => selected.rebase(
            execution, runtime, snapshot, retired, producer, maximum, admission, control,
        ),
        Preparation::Pending { .. } | Preparation::FullPayload { .. } => {
            unreachable!("native candidate selection precedes publication")
        }
    }
}
