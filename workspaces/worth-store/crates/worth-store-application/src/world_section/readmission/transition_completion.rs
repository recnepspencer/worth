//! One actual completed original chain permits one guarded descriptor promotion.
use super::*;
use crate::descriptor::transition_binding::PreparedHomeTransitionBinding;
use crate::open::readmission::transition::{self, ApplicationHomeTransitionCompletionDenial};
use worth_relational::facade::runtime::{
    PreparedRelationalHomeProgramContracts, RelationalHomeProgramPromotionRefused,
    RelationalInitialSchemaInstallationReceipt,
};
use worth_runtime_world::facade::{PerformedCompositePublication, ProductBranchObservation};

#[derive(Debug)]
pub enum ApplicationHomeTransitionBindingStageDenial {
    World(worth_runtime_world::facade::WorldHomeProgramBindingDenial),
    Descriptor(crate::descriptor::transition_binding::ApplicationHomeTransitionBindingRefused),
    WorldAfterBinding {
        cause: worth_runtime_world::facade::WorldHomeProgramBindingDenial,
        acknowledgement: worth_proof::HomeProgramTransitionBindingCommitted,
    },
}
#[derive(Debug)]
pub enum ApplicationHomeTransitionFinishDenial {
    Completion(ApplicationHomeTransitionCompletionDenial),
    Intent(crate::ApplicationHomeTransitionIntentDenial),
    Descriptor(crate::ApplicationHomeDescriptorDenial),
    ReceiverConfigurationRequired,
    Promotion(RelationalHomeProgramPromotionRefused<ApplicationHomeTransitionBindingStageDenial>),
}

impl<D, I, E, Ctx, T> WorldReadmittedApplicationHome<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    E: Send + Sync + 'static,
    Ctx: Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    /// Recorded original installation and current target, sealed by this same
    /// Home owner. This grants no initialization or program-transition step.
    pub fn home_program_transition_installation_binding(
        &self,
    ) -> Result<
        Option<
            worth_proof::AdmittedHomeProgramTransitionSuccessor<
                worth_foundational::facade::HomeProgramTransitionInstallationBinding,
            >,
        >,
        crate::ApplicationHomeTransitionIntentDenial,
    > {
        self.home
            .context
            .transition
            .installation_binding(&self.home.context.descriptor.binding)
    }

    pub fn accept_home_program_transition_step(
        &mut self,
        performed: PerformedCompositePublication,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), crate::ApplicationHomeTransitionStepRejected> {
        transition::completion::accept(
            &mut self.home.context,
            self.world.owner_identity(),
            performed,
            request,
        )
    }

    pub fn finish_home_program_transition(
        &mut self,
        source: &ProductBranchObservation,
        target: &PreparedRelationalHomeProgramContracts,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<RelationalInitialSchemaInstallationReceipt, ApplicationHomeTransitionFinishDenial>
    {
        use ApplicationHomeTransitionBindingStageDenial as Stage;
        use ApplicationHomeTransitionFinishDenial as Denial;
        let context = &mut self.home.context;
        let completed = transition::completion::completed(context, source, request)
            .map_err(Denial::Completion)?;
        let intent = context
            .transition
            .committed_intent(&context.descriptor.binding)
            .map_err(Denial::Intent)?
            .ok_or(Denial::ReceiverConfigurationRequired)?;
        let installation = context.transition.successor_installation(intent);
        let required = context
            .transition
            .required()
            .ok_or(Denial::ReceiverConfigurationRequired)?;
        let configuration = target
            .receiver_configuration()
            .ok_or(Denial::ReceiverConfigurationRequired)?;
        let prepared = PreparedHomeTransitionBinding::prepare(
            context,
            intent.successor(),
            configuration,
            request,
        )
        .map_err(Denial::Descriptor)?;
        let successor = context.transition.admit_successor(intent.successor());
        let plan = intent.plan().commitment();
        let world = self.world.branch_port();
        let mut actual_acknowledgement = None;
        let receipt = self
            .home
            .relational
            .runtime()
            .promote_home_program_after_binding(
                request,
                target,
                source.basis().relational_basis(),
                &completed,
                || {
                    world
                        .with_home_program_binding(
                            request,
                            source,
                            &required,
                            &successor,
                            |world| {
                                let acknowledgement = prepared
                                    .publish(
                                        &context.serving,
                                        &mut context.descriptor,
                                        &context.transition,
                                        plan,
                                        request,
                                    )
                                    .map_err(Stage::Descriptor)?;
                                if let Err(cause) = world.acknowledge(&acknowledgement) {
                                    return Err(Stage::WorldAfterBinding {
                                        cause,
                                        acknowledgement,
                                    });
                                }
                                actual_acknowledgement = Some(acknowledgement.clone());
                                Ok(acknowledgement)
                            },
                        )
                        .map_err(Stage::World)?
                },
            )
            .map_err(Denial::Promotion)?;
        context.transition.finish_binding(
            actual_acknowledgement
                .expect("a successful original promotion consumed the actual acknowledgement"),
            target.catalog_receipt(),
            installation,
        );
        Ok(receipt)
    }
}
