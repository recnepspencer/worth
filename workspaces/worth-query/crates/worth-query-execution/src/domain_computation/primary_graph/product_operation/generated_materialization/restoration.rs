use crate::domain_computation::primary_graph::product_operation::generated_materialization::restoration::in_phase::InPhaseParentOps;
mod in_phase;
mod publication;
use worth_query_installation::facade::ApplicationSchema;
use worth_relational::facade::branch::RelationalRematerializationCompletion;

pub(crate) mod failure;
mod invariant_admission;
mod invariant_denial;
pub(in crate::domain_computation::primary_graph) use invariant_admission::admit_required_invariants;
mod no_effect;
mod receipt;
mod recovery;
mod republication;
mod settlement;

pub use failure::WorthQueryGeneratedOutputRestorationFailureCause;
use failure::{preparation_failure_cause, restoration_failure};
pub use invariant_denial::WorthQueryGeneratedOutputInvariantAdmissionDenial;

pub use no_effect::{
    WorthQueryGeneratedOutputPublicationNoEffect, WorthQueryGeneratedOutputPublicationNoEffectCause,
};
pub use receipt::WorthQueryGeneratedOutputRestorationReceipt;
pub use recovery::{
    WorthQueryGeneratedOutputRestorationRecovery,
    WorthQueryGeneratedOutputRestorationRecoveryFailure,
    WorthQueryGeneratedOutputRestorationRecoveryStage,
};

use super::{WorthQueryCompletedGeneratedOutputReconstruction, WorthQuerySuspendedGeneratedOutput};
use crate::domain_computation::execution_runtime::product_world::WorthQueryProductPublicationBinding;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationOutputCorrespondence, WorthQueryApplicationProducerBinding,
};

/// A generated output restored to its product branch, with the restoration
/// receipt.
///
/// Returned by `restore_generated_output`, or by
/// `continue_generated_output_restoration_recovery` once a recovered
/// restoration is published.
pub struct WorthQueryRestoredGeneratedOutput {
    branch: crate::basis::WorthQueryProductBranch,
    commit: WorthQueryGeneratedOutputRestorationReceipt,
}

impl WorthQueryRestoredGeneratedOutput {
    pub fn product_branch(&self) -> crate::basis::WorthQueryProductBranch {
        self.branch
    }

    pub fn commit(&self) -> &WorthQueryGeneratedOutputRestorationReceipt {
        &self.commit
    }
}

/// Why restoring a generated output did not complete.
pub enum WorthQueryGeneratedOutputRestorationFailure {
    /// Nothing was published. The suspended output is handed back so it can be
    /// reconstructed again; the cause says why.
    Rejected {
        suspended: WorthQuerySuspendedGeneratedOutput,
        cause: WorthQueryGeneratedOutputRestorationFailureCause,
    },
    /// Some owners moved, but the product head did not. Continue the recovery
    /// this carries.
    ProductUnpublished(WorthQueryUnpublishedGeneratedOutputRestoration),
}

impl WorthQueryGeneratedOutputRestorationFailure {
    pub fn cause(&self) -> Option<&WorthQueryGeneratedOutputRestorationFailureCause> {
        match self {
            Self::Rejected { cause, .. } => Some(cause),
            Self::ProductUnpublished(_) => None,
        }
    }

    pub fn into_suspended(self) -> Result<WorthQuerySuspendedGeneratedOutput, Self> {
        match self {
            Self::Rejected { suspended, .. } => Ok(suspended),
            unpublished @ Self::ProductUnpublished(_) => Err(unpublished),
        }
    }
}

/// A restoration whose publication moved some owners but not the product head.
///
/// Turn it into a recovery with `into_recovery`, then continue with
/// `continue_generated_output_restoration_recovery`.
#[must_use = "unpublished restoration custody must remain with its World recovery authority"]
pub struct WorthQueryUnpublishedGeneratedOutputRestoration {
    product: crate::domain_computation::WorthQueryProductUnpublishedApplication,
    retry: recovery::RestorationRetry,
}

impl WorthQueryUnpublishedGeneratedOutputRestoration {
    fn into_parts(
        self,
    ) -> (
        crate::domain_computation::WorthQueryProductUnpublishedApplication,
        recovery::RestorationRetry,
    ) {
        (self.product, self.retry)
    }

    pub fn into_recovery(self) -> WorthQueryGeneratedOutputRestorationRecovery {
        WorthQueryGeneratedOutputRestorationRecovery::from_unpublished(self)
    }
}

pub trait RestorationOps<Schema>:
    crate::domain_computation::primary_graph::product_operation::host::ProductOperationHost<Schema>
where
    Schema: ApplicationSchema,
{
    fn restore_generated_output<Producer>(
        &self,
        completed: WorthQueryCompletedGeneratedOutputReconstruction<Schema, Producer>,
        request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    ) -> Result<WorthQueryRestoredGeneratedOutput, WorthQueryGeneratedOutputRestorationFailure>
    where
        Producer: WorthQueryApplicationProducerBinding<Schema>,
    {
        let mut retained = Some(completed);
        let mut completed_result = None;
        let result = self
            .application_runtime()
            .with_application_advancement(request, |phase| {
                completed_result = Some(self.restore_generated_output_in_phase(
                    retained.take().expect("one admitted restoration"),
                    &phase,
                    request,
                ));
            });
        // The callback owns the actual outcome after it consumes custody. A
        // final outer control refusal cannot discard that outcome or recovery.
        if let Some(result) = completed_result {
            return result;
        }
        match result {
            Ok(()) => unreachable!("advancement invokes its callback"),
            Err(cause) => Err(restoration_failure(
                retained.expect("refusal precedes admission").suspended,
                WorthQueryGeneratedOutputRestorationFailureCause::advancement(cause),
            )),
        }
    }
}

impl<
        Schema,
        HostRuntime: crate::domain_computation::primary_graph::product_operation::host::ProductOperationHost<
                Schema,
            > + ?Sized,
    > RestorationOps<Schema> for HostRuntime
where
    Schema: ApplicationSchema,
{
}

pub(super) trait RestorationParentOps<Schema>:
    crate::domain_computation::primary_graph::product_operation::host::ProductOperationHost<Schema>
where
    Schema: ApplicationSchema,
{
    fn materialization_publication_binding(
        &self,
        observation: worth_runtime_world::facade::ProductBranchObservation,
    ) -> WorthQueryProductPublicationBinding {
        self.product_runtime().publication_binding(observation)
    }

    fn unpublished_materialization(
        &self,
        effects: worth_runtime_world::facade::ProductUnpublishedOwnerEffects,
        product: &crate::basis::WorthQueryProductBranchLease,
    ) -> crate::domain_computation::WorthQueryProductUnpublishedApplication {
        self.unpublished_materialization_from_binding(effects, &product.publication_binding())
    }

    fn unpublished_materialization_from_binding(
        &self,
        effects: worth_runtime_world::facade::ProductUnpublishedOwnerEffects,
        publication: &WorthQueryProductPublicationBinding,
    ) -> crate::domain_computation::WorthQueryProductUnpublishedApplication {
        crate::domain_computation::WorthQueryProductUnpublishedApplication::new(
            effects,
            publication.recovery(),
            self.primary_provider()
                .unpublished_idempotency_disposition(),
        )
    }
}

impl<
        Schema,
        HostRuntime: crate::domain_computation::primary_graph::product_operation::host::ProductOperationHost<
                Schema,
            > + ?Sized,
    > RestorationParentOps<Schema> for HostRuntime
where
    Schema: ApplicationSchema,
{
}

fn suspended_from_completion(
    publication: WorthQueryProductPublicationBinding,
    branch: crate::basis::WorthQueryProductBranch,
    correspondence: std::sync::Arc<WorthQueryApplicationOutputCorrespondence>,
    producer: super::ProducerQualification,
    completion: RelationalRematerializationCompletion,
) -> WorthQuerySuspendedGeneratedOutput {
    WorthQuerySuspendedGeneratedOutput {
        publication,
        branch,
        custody: completion.into_custody(),
        correspondence,
        producer,
    }
}
pub use recovery::AdvancementOps;
pub use republication::RepublicationDoorsOps;
