//! A pending original intent resumes at its first nondurable step only.
use super::WorldReadmittedApplicationHome;
use crate::open::readmission::transition::{self, DurableHomeTransitionPrefix};
use crate::{AdmittedApplicationHomeProgramStep, ApplicationHomeTransitionIntentDenial};
use worth_execution::ExecutionRequest;
use worth_foundational::facade::{
    DurableCompositePublicationAttemptIdentity, HomeProgramTransitionBodyDigest,
    HomeProgramTransitionPlanDigest, HomeProgramTransitionTargetMeaning,
};
use worth_runtime_world::facade::{
    ProductBranchObservation, RuntimeWorldRecoveryDenial, RuntimeWorldServiceDenial,
    WorldRecoveredUnpublishedEffects,
};

/// The next original step of a readmitted pending transition. Durable steps
/// are never offered; each variant carries the only permission its state
/// admits. World classifies a recovered original by its Native custody.
pub enum ApplicationHomeProgramTransitionResumption {
    /// Every original step's World occurrence is durable; the binding remains.
    StepsDurable,
    /// No durable World open exists; the grant carries its one execution.
    Unopened {
        step: u32,
        grant: AdmittedApplicationHomeProgramStep,
    },
    /// The step's original World open is recovered custody. It is continued
    /// or resumed from that original, never replaced by a fresh attempt.
    Recovered {
        step: u32,
        grant: AdmittedApplicationHomeProgramStep,
        claim: WorldRecoveredUnpublishedEffects,
    },
}

#[derive(Debug)]
pub enum ApplicationHomeTransitionResumptionDenial {
    Intent(ApplicationHomeTransitionIntentDenial),
    Inventory(RuntimeWorldServiceDenial<RuntimeWorldRecoveryDenial>),
    RecoveredOriginal(RuntimeWorldRecoveryDenial),
}

impl<D, I, E, Ctx, T> WorldReadmittedApplicationHome<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    E: Send + Sync + 'static,
    Ctx: Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    /// The caller's recomputed plan must equal the recorded intent
    /// (`TransitionPlanDrift`, or `TransitionPending` for another successor)
    /// before the durable prefix at the actual head `expected` is read.
    pub fn resume_home_program_transition(
        &self,
        expected: &ProductBranchObservation,
        digest: HomeProgramTransitionPlanDigest,
        steps: u32,
        target: HomeProgramTransitionTargetMeaning,
        bodies: &[HomeProgramTransitionBodyDigest],
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<ApplicationHomeProgramTransitionResumption, ApplicationHomeTransitionResumptionDenial>
    {
        use ApplicationHomeProgramTransitionResumption as Resumption;
        use ApplicationHomeTransitionResumptionDenial as Denial;
        let context = &self.home.context;
        let owner = self.world.owner_identity();
        let prefix = transition::durable_prefix(
            context, owner, expected, digest, steps, target, bodies, request,
        )
        .map_err(Denial::Intent)?;
        let DurableHomeTransitionPrefix::Next { step, original } = prefix else {
            return Ok(Resumption::StepsDurable);
        };
        match self.recovered_original(original)? {
            Some(claim) => {
                let grant = transition::admit_readmitted_step(
                    context, owner, step, &claim, digest, steps, target, bodies, request,
                )
                .map_err(Denial::Intent)?;
                Ok(Resumption::Recovered { step, grant, claim })
            }
            None => {
                let grant = transition::admit_unopened_step(
                    context, owner, step, expected, digest, steps, target, bodies, request,
                )
                .map_err(Denial::Intent)?;
                Ok(Resumption::Unopened { step, grant })
            }
        }
    }

    /// World's recovered custody of exactly this original attempt, if any.
    /// Every other inspected claim is dropped back to World unchanged.
    fn recovered_original(
        &self,
        original: DurableCompositePublicationAttemptIdentity,
    ) -> Result<Option<WorldRecoveredUnpublishedEffects>, ApplicationHomeTransitionResumptionDenial>
    {
        use ApplicationHomeTransitionResumptionDenial as Denial;
        let inspection = self.world.inspection_port();
        let recovery = self.world.recovery_port();
        let page_size = std::num::NonZeroUsize::new(64).expect("nonzero recovery page");
        let mut after = None;
        loop {
            let page = inspection
                .recovery_page(after.as_ref(), page_size)
                .map_err(Denial::Inventory)?;
            for row in page.rows() {
                let claim = recovery
                    .inspect_recovered_effects(row.handle())
                    .map_err(Denial::RecoveredOriginal)?;
                let attempt = claim.attempt_identity();
                if attempt.ordinal() == original.ordinal()
                    && attempt.owner_identity().durable() == Some(original.owner())
                {
                    return Ok(Some(claim));
                }
            }
            match page.next_after() {
                Some(next) => after = Some(next.clone()),
                None => return Ok(None),
            }
        }
    }
}
