//! A durable Home's performed witness, rejoined from its retained native effect
//! and World original history, so it survives a reopen exactly as captured.
use super::super::correction_witness::{retain, run, CorrectionPerformedWitness};
use super::super::correction_witness::CorrectionWitnessDenial as Denial;
use super::super::WorthQueryCanonicalCorrectionFact as Fact;
use super::WorthQueryPrimaryGraphProvider;
use crate::domain_computation::application_aftermath::{
    WorthQueryAftermathCausalRole as Role, WorthQueryCommittedAftermathCausality as Sealed,
    WorthQueryCorrectionLineageBinding as Lineage, WorthQueryCorrectionSourceCoordinates,
    WorthQueryRecordedCorrectionSource as Recorded,
};
use std::sync::Arc;
use worth_execution::ExecutionRequest;
use worth_foundational::facade::WorldOwnerCoordinate;
use worth_query_declaration::archive_decode_authority::ARCHIVE_DECODE;
use worth_relational::facade::durability::RelationalAssociatedEffectSubject as Subject;
use worth_relational::facade::history::RelationalCommitReceipt;
use worth_relational::facade::runtime::RelationalRuntime;
use worth_runtime_world::facade::{
    CompositeCommitIdentity, RuntimeWorldOwnerIdentity, WorldProgramActivationHistory,
};

/// One durable Home's retained originals and the native runtime they join.
pub(crate) struct DurableOriginals<'a> {
    pub(crate) originals: &'a WorldProgramActivationHistory,
    pub(crate) owner: RuntimeWorldOwnerIdentity,
    pub(crate) native: &'a RelationalRuntime,
}

impl WorthQueryPrimaryGraphProvider {
    /// The witness a durable Home's session captured, rebuilt from durable
    /// custody alone. `None` means the occurrence has no published original.
    pub fn restore_correction_witness(
        &self,
        execution: ExecutionRequest<'_, '_>,
        originals: &WorldProgramActivationHistory,
        owner: RuntimeWorldOwnerIdentity,
        receipt: &RelationalCommitReceipt,
    ) -> Result<Option<Arc<CorrectionPerformedWitness>>, Denial> {
        let restored = self.graph.with_runtime(&execution, |native| {
            let home = DurableOriginals {
                originals,
                owner,
                native,
            };
            let Some((world, fact)) = self.durable_fact(execution, &home, receipt)? else {
                return Ok(None);
            };
            match fact {
                None => Ok(Some((world, None))),
                Some(fact) => Ok(home
                    .seal(execution, receipt, fact)?
                    .map(|sealed| (world, Some(sealed)))),
            }
        })?;
        let Some((world, sealed)) = restored else {
            return Ok(None);
        };
        run(execution, |context| {
            retain(execution, context, &world, sealed.as_ref())
        })
        .map(Some)
    }

    /// The occurrence's own World commit and its decoded canonical fact, if any.
    pub(crate) fn durable_fact(
        &self,
        execution: ExecutionRequest<'_, '_>,
        home: &DurableOriginals<'_>,
        receipt: &RelationalCommitReceipt,
    ) -> Result<Option<(CompositeCommitIdentity, Option<Fact>)>, Denial> {
        let Some((subject, world)) = home.rejoin(execution, receipt)? else {
            return Ok(None);
        };
        let effect = home
            .native
            .durability_authority()
            .readmit_associated_effect(&execution, &subject)
            .map_err(|cause| Denial::OriginalSource(Box::new(cause)))?;
        let fact = self
            .readmitted_original_causality(&execution, home.native, &effect, home.coordinate()?)
            .map_err(|_| Denial::InconsistentCausality)?;
        Ok(Some((world, fact)))
    }
}

impl DurableOriginals<'_> {
    fn coordinate(&self) -> Result<WorldOwnerCoordinate, Denial> {
        let owner = self
            .owner
            .durable()
            .ok_or(Denial::OriginalHistory(worth_runtime_world::facade::WorldReadmissionDenial::ForeignOwner))?;
        Ok(WorldOwnerCoordinate::Home { owner })
    }

    /// The native effect committed at `receipt` and the published World commit
    /// whose occurrence binds exactly that effect; `None` when either is absent.
    fn rejoin(
        &self,
        execution: ExecutionRequest<'_, '_>,
        receipt: &RelationalCommitReceipt,
    ) -> Result<Option<(Subject, CompositeCommitIdentity)>, Denial> {
        let Some(subject) = self
            .native
            .durability_authority()
            .discover_associated_effect(&execution, receipt)
            .map_err(|cause| Denial::OriginalSource(Box::new(cause)))?
        else {
            return Ok(None);
        };
        let world = self
            .originals
            .resolve_published_native_commit(self.owner, &subject)
            .map_err(Denial::OriginalHistory)?;
        Ok(world.map(|world| (subject, world)))
    }

    /// One recorded coordinate, rejoined to its own published World commit.
    fn recorded(
        &self,
        execution: ExecutionRequest<'_, '_>,
        coordinates: &WorthQueryCorrectionSourceCoordinates,
    ) -> Result<Option<Recorded>, Denial> {
        if coordinates.world_owner() != self.coordinate()? {
            return Ok(None);
        }
        Ok(self
            .rejoin(execution, coordinates.relational())?
            .filter(|(_, world)| world.ordinal() == coordinates.world_commit())
            .map(|(_, world)| {
                Recorded::from_untrusted_parts(
                    ARCHIVE_DECODE,
                    world,
                    coordinates.relational().clone(),
                    coordinates.member(),
                )
            }))
    }

    /// Seals `fact` only when every coordinate rejoins its own published original.
    pub(crate) fn seal(
        &self,
        execution: ExecutionRequest<'_, '_>,
        receipt: &RelationalCommitReceipt,
        fact: Fact,
    ) -> Result<Option<Sealed>, Denial> {
        if fact.child != *receipt {
            return Err(Denial::InconsistentCausality);
        }
        let Some(source) = self.recorded(execution, &fact.source)? else {
            return Ok(None);
        };
        let Some(root) = self.recorded(execution, &fact.root)? else {
            return Ok(None);
        };
        let lineage = match (fact.role, &fact.selected, &fact.active) {
            (Role::RecordedReverse, None, None) => {
                let prior = match &fact.prior {
                    None => None,
                    Some(prior) => match self.recorded(execution, prior)? {
                        None => return Ok(None),
                        prior => prior,
                    },
                };
                Lineage::reverse(root, prior)
            }
            (Role::RecordedReapply, Some(selected), Some(active)) => {
                let Some(selected) = self.recorded(execution, selected)? else {
                    return Ok(None);
                };
                let Some(active) = self.recorded(execution, active)? else {
                    return Ok(None);
                };
                Lineage::reapply(root, selected, active)
            }
            _ => return Err(Denial::InconsistentCausality),
        };
        Sealed::seal_parts(
            fact.role,
            fact.parent,
            fact.child.branch_id.clone(),
            source,
            lineage,
            fact.child,
            fact.record,
        )
        .map(Some)
        .ok_or(Denial::InconsistentCausality)
    }
}

pub mod court_doors_for_test;
