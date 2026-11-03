//! Attached Home root publication borrows the same sole lifecycle owner.
use super::*;
use crate::branch::{
    ProductBranchCreationIntent, RuntimeWorldBootstrapIntent, RuntimeWorldBootstrapNoEffectCause,
    WorldDurableBootstrapOutcome as Outcome, WorldDurableBootstrapRefusal as Refusal,
};
use worth_relational::facade::branch::AdmittedRelationalBranchBasis;
use worth_signal::facade::branch::AdmittedSignalBranchBasis;
impl<D, I, E, Ctx, T> RuntimeWorldPublicationPort<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    E: Send + Sync + 'static,
    Ctx: Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    /// Bootstrap the actual current component tuple using this Home's private
    /// Bridge baseline. No Bridge or lifecycle capability leaves the Home.
    pub fn begin_durable_bootstrap_root(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        creation: ProductBranchCreationIntent,
        relational: AdmittedRelationalBranchBasis,
        signal: AdmittedSignalBranchBasis,
    ) -> Outcome {
        let owner = match self.service() {
            Ok(owner) => owner,
            Err(_) => return unavailable(),
        };
        if owner.state.durable.is_none() {
            return Outcome::Refused(Refusal::NotHomeBound);
        }
        if let Err(cause) = execution.consult() {
            return Outcome::Refused(Refusal::Medium(
                crate::durability::WorldDurableMediumDenial::Request(cause),
            ));
        }
        if owner
            .state
            .signal
            .basis_port()
            .compare_current_exact(&signal)
            .is_err()
        {
            return Outcome::Refused(Refusal::Preparation(
                RuntimeWorldBootstrapNoEffectCause::ForeignBasis,
            ));
        }
        let Some(graph) = signal
            .observation()
            .target()
            .as_basis()
            .and_then(|basis| basis.graph_instance_id().parse::<u64>().ok())
        else {
            return Outcome::Refused(Refusal::Preparation(
                RuntimeWorldBootstrapNoEffectCause::ForeignBasis,
            ));
        };
        let correspondence = owner
            .state
            .bridge
            .as_ref()
            .expect("serving World owns correspondence")
            .admit_runtime_baseline(graph);
        owner.begin_durable_bootstrap(
            execution,
            RuntimeWorldBootstrapIntent::new(creation, relational, signal, correspondence),
        )
    }
    /// Advance only the owner's retained original bootstrap. No replacement
    /// component inputs or identities can be paired with its completion.
    pub fn advance_durable_bootstrap_root(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Outcome {
        match self.service() {
            Ok(owner) => owner.advance_durable_bootstrap(execution),
            Err(_) => unavailable(),
        }
    }
}
fn unavailable() -> Outcome {
    Outcome::RecoveryRequired(Refusal::Preparation(
        RuntimeWorldBootstrapNoEffectCause::OwnerUnavailable,
    ))
}
