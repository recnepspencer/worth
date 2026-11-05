//! Move the same concrete Home only after its persisted owner admission.
use super::WorldReadmittedApplicationHome;
use crate::open::readmission::transition::{
    readmit_no_pending_owners, require_no_pending_home_journals,
    ApplicationHomeOwnerAdmissionDenial,
};
use worth_execution::ExecutionRequest;

/// Store retains the actual Home, incarnation and lease through admission.
/// Descriptive data cannot construct this carrier.
///
/// ```compile_fail,E0451
/// use worth_store_application::{NoPendingApplicationHomeTransition, WorldReadmittedApplicationHome};
/// fn forge(home: WorldReadmittedApplicationHome<(), (), (), (), ()>) {
///     let _ = NoPendingApplicationHomeTransition { home };
/// }
/// ```
pub struct NoPendingApplicationHomeTransition {
    home: WorldReadmittedApplicationHome<(), (), (), (), ()>,
}

impl NoPendingApplicationHomeTransition {
    /// The closed Host completion consumes the actual admitted owner once.
    ///
    /// ```
    /// use worth_store_application::{NoPendingApplicationHomeTransition, WorldReadmittedApplicationHome};
    /// fn once(admitted: NoPendingApplicationHomeTransition) -> WorldReadmittedApplicationHome<(), (), (), (), ()> {
    ///     admitted.into_no_pending_home()
    /// }
    /// ```
    ///
    /// ```compile_fail,E0382
    /// use worth_store_application::NoPendingApplicationHomeTransition;
    /// fn twice(admitted: NoPendingApplicationHomeTransition) {
    ///     let _home = admitted.into_no_pending_home();
    ///     let _again = admitted.into_no_pending_home();
    /// }
    /// ```
    pub fn into_no_pending_home(self) -> WorldReadmittedApplicationHome<(), (), (), (), ()> {
        self.home
    }
}

impl<D, I, E, Ctx, T> WorldReadmittedApplicationHome<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    E: Send + Sync + 'static,
    Ctx: Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    /// Claim an original intent assignment for the World's protected absence
    /// check. This is insufficient to repeat an already durable open or effect.
    pub fn admit_unopened_home_program_transition_step(
        &self,
        step: u32,
        expected: &worth_runtime_world::facade::ProductBranchObservation,
        digest: worth_foundational::facade::HomeProgramTransitionPlanDigest,
        steps: u32,
        target: worth_foundational::facade::HomeProgramTransitionTargetMeaning,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<
        crate::AdmittedApplicationHomeProgramStep,
        crate::ApplicationHomeTransitionIntentDenial,
    > {
        crate::open::readmission::transition::admit_unopened_step(
            &self.home.context,
            self.world.owner_identity(),
            step,
            expected,
            digest,
            steps,
            target,
            request,
        )
    }
    /// Admit an original recovered step only after recomputing its exact plan.
    /// The World claim supplies its genuine current-incarnation source identity.
    pub fn admit_readmitted_home_program_transition_step(
        &self,
        step: u32,
        claim: &worth_runtime_world::facade::WorldRecoveredUnpublishedEffects,
        digest: worth_foundational::facade::HomeProgramTransitionPlanDigest,
        steps: u32,
        target: worth_foundational::facade::HomeProgramTransitionTargetMeaning,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<
        crate::AdmittedApplicationHomeProgramStep,
        crate::ApplicationHomeTransitionIntentDenial,
    > {
        crate::open::readmission::transition::admit_readmitted_step(
            &self.home.context,
            self.world.owner_identity(),
            step,
            claim,
            digest,
            steps,
            target,
            request,
        )
    }

    /// Admit a single original step only after its intent is actually committed.
    /// An abandoned original open is recovered, never replaced by another token.
    pub fn admit_home_program_transition_step(
        &self,
        step: u32,
        expected: &worth_runtime_world::facade::ProductBranchObservation,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<
        crate::AdmittedApplicationHomeProgramStep,
        crate::ApplicationHomeTransitionIntentDenial,
    > {
        crate::open::readmission::transition::admit_step(
            &self.home.context,
            step,
            expected,
            request,
        )
    }

    /// Stage the complete original intent without yet starting a journal write.
    /// Only Query's genuinely installed successor binding enters this owner.
    pub fn stage_home_program_transition_intent(
        &mut self,
        reserved: crate::ReservedApplicationHomeTransitionPlan,
        successor: worth_proof::AdmittedApplicationHomeBinding<
            worth_query_installation::facade::WorthQueryHomeBindingAdmission,
        >,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), crate::ApplicationHomeTransitionIntentDenial> {
        crate::open::readmission::transition::stage_intent(
            &self.home.context,
            self.home.relational.runtime(),
            self.world.owner_identity(),
            reserved,
            successor,
            request,
        )
    }

    pub fn advance_home_program_transition_intent(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<
        crate::ApplicationHomeTransitionIntentOutcome,
        crate::ApplicationHomeTransitionIntentDenial,
    > {
        crate::open::readmission::transition::advance_intent(&self.home.context, request)
    }

    pub fn transition_admission_observation(
        &self,
    ) -> crate::ApplicationHomeTransitionAdmissionObservation {
        self.home.context.transition.observation()
    }

    /// Borrow the staged original meaning; this observation declares neither
    /// intent commitment nor migration progress.
    pub fn home_program_transition_intent(&self) -> Option<&crate::HomeProgramTransitionIntent> {
        self.home.context.transition.intent()
    }

    /// Reserve original World attempt assignments for a complete plan. This
    /// does not publish an intent, a World open, a Native step or a new head.
    pub fn reserve_home_program_transition_plan(
        &self,
        expected: worth_runtime_world::facade::ProductBranchObservation,
        digest: worth_foundational::facade::HomeProgramTransitionPlanDigest,
        steps: u32,
        target: worth_foundational::facade::HomeProgramTransitionTargetMeaning,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<
        crate::ReservedApplicationHomeTransitionPlan,
        crate::ApplicationHomeTransitionPlanningDenial,
    > {
        crate::open::readmission::transition::reserve_plan(
            &self.home.context,
            self.world.publication_port(),
            self.home.relational.runtime(),
            expected,
            digest,
            steps,
            target,
            request,
        )
    }
}

pub struct ApplicationHomeOwnerAdmissionRefused {
    home: Box<WorldReadmittedApplicationHome<(), (), (), (), ()>>,
    cause: ApplicationHomeOwnerAdmissionDenial,
}
impl ApplicationHomeOwnerAdmissionRefused {
    pub fn reason(&self) -> &ApplicationHomeOwnerAdmissionDenial {
        &self.cause
    }
    pub fn into_home(self) -> WorldReadmittedApplicationHome<(), (), (), (), ()> {
        *self.home
    }
    pub fn into_parts(
        self,
    ) -> (
        WorldReadmittedApplicationHome<(), (), (), (), ()>,
        ApplicationHomeOwnerAdmissionDenial,
    ) {
        (*self.home, self.cause)
    }
}
impl std::fmt::Debug for ApplicationHomeOwnerAdmissionRefused {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ApplicationHomeOwnerAdmissionRefused")
            .field("home", &self.home.home_identity())
            .field("cause", &self.cause)
            .finish()
    }
}

impl WorldReadmittedApplicationHome<(), (), (), (), ()> {
    pub fn readmit_no_pending_owners(
        self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<NoPendingApplicationHomeTransition, ApplicationHomeOwnerAdmissionRefused> {
        let checked = if self
            .home
            .relational
            .world_fresh_source_admission()
            .is_some()
        {
            // The actual fresh source was admitted before World/bootstrap writes.
            // Its initial-root proof cannot be revalidated after lawful advance.
            require_no_pending_home_journals(&self.home.context, request)
        } else if let Some(recovered) = self.home.relational.world_recovery_authority() {
            readmit_no_pending_owners(&self.home.context, recovered, request)
        } else {
            Err(ApplicationHomeOwnerAdmissionDenial::NativeRecoveryRequired)
        };
        match checked {
            Ok(()) => Ok(NoPendingApplicationHomeTransition { home: self }),
            Err(cause) => Err(ApplicationHomeOwnerAdmissionRefused {
                home: Box::new(self),
                cause,
            }),
        }
    }
}
