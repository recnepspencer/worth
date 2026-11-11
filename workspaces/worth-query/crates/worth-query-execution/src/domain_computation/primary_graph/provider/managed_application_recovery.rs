use crate::domain_computation::execution_runtime::product_world::publication_denial::{
    WorthQueryProductPublicationDenial as PublicationDenial,
    WorthQueryProductPublicationOutcome as PublicationOutcome,
};
use crate::domain_computation::primary_graph::application_contribution::advancement_denial;
use worth_query_installation::facade::ApplicationSchema;
use worth_runtime_world::facade::{
    RuntimeWorldRecoveryDenial, RuntimeWorldSettledRelationalAdoptionDenial,
};

use super::{publish_recovered, unpublished_idempotency::ManagedUnpublishedRecoveryStop};
use crate::domain_computation::primary_graph::{
    application_attempt::WorthQueryApplicationIdempotencyBinding,
    WorthQueryAdmittedApplicationOperation, WorthQueryApplicationIdempotencyResolutionDenial,
    WorthQueryOperationAuthorizationDenial, WorthQueryOutputDemandDenial,
    WorthQueryPrimaryGraphApplicationRuntime,
};
use crate::domain_computation::{
    WorthQueryProductUnpublishedRecovery, WorthQueryProductUnpublishedRecoveryReleaseFailure,
};

mod admission;
mod performed;
mod producer;
pub(in crate::domain_computation::primary_graph) use producer::ProducerRecoveryCustody;
mod publication;
pub use performed::WorthQueryManagedApplicationRecoveryPerformed;

enum RecoveredPublicationDelivery<'a> {
    Ordinary,
    ProgramSource,
    OwningSource,
    Producer(&'a mut dyn FnMut(ProducerRecoveryCustody)),
}

/// Owner result of an admitted recovery: performed publication, no effect, or
/// a new exact partial recovery replacing the previously retained handle.
pub enum WorthQueryManagedApplicationRecoveryOutcome {
    Performed(WorthQueryManagedApplicationRecoveryPerformed),
    /// World made no movement. The supplied exact recovery remains retryable.
    NoEffect,
    /// World issued a new partial; its provider claim was staged before the
    /// owner effect and now replaces the original exact recovery handle.
    ProductUnpublished {
        next: WorthQueryProductUnpublishedRecovery,
        prior_cleanup_failure: Option<WorthQueryProductUnpublishedRecoveryReleaseFailure>,
    },
}

/// A refusal before another World owner effect. The borrowed original
/// recovery remains retained by its provider entry for a later admitted try.
#[derive(Debug)]
pub enum WorthQueryManagedApplicationRecoveryDenial {
    ExecutionDenied(crate::domain_computation::primary_graph::WorthQueryAdvancementDenial),
    Authorization(WorthQueryOperationAuthorizationDenial),
    ForeignAdmission,
    BindingMismatch,
    ProviderBusy,
    ProviderCapacity,
    ProviderAdmission(worth_relational::facade::mvcc::CompanionPreflightStop),
    World(RuntimeWorldRecoveryDenial),
    ProductUnavailable,
    Bridge(worth_runtime_bridge::facade::BridgeConditionalDenial),
    /// Rich World head-comparison evidence is retained only for this refusal.
    Adoption(Box<RuntimeWorldSettledRelationalAdoptionDenial>),
    Demand(WorthQueryOutputDemandDenial),
}

impl From<ManagedUnpublishedRecoveryStop> for WorthQueryManagedApplicationRecoveryDenial {
    fn from(stop: ManagedUnpublishedRecoveryStop) -> Self {
        match stop {
            ManagedUnpublishedRecoveryStop::Missing | ManagedUnpublishedRecoveryStop::Busy => {
                Self::ProviderBusy
            }
            ManagedUnpublishedRecoveryStop::Capacity => Self::ProviderCapacity,
        }
    }
}

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema,
{
    /// Continues the exact retained application partial with a fresh
    /// admitted operation. It never reruns the Relational candidate or handler.
    pub fn recover_admitted_unpublished_application<Operation, Input, Scope>(
        &self,
        recovery: &WorthQueryProductUnpublishedRecovery,
        admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> Result<
        WorthQueryManagedApplicationRecoveryOutcome,
        WorthQueryManagedApplicationRecoveryDenial,
    >
    where
        Input: Clone + Send + Sync + 'static,
    {
        if !admission.belongs_to(
            self.runtime.authority_identity(),
            &self.installed_schema.binding_identity(),
        ) {
            return Err(WorthQueryManagedApplicationRecoveryDenial::ForeignAdmission);
        }
        admission
            .validate_installed_operation(&self.runtime.installed_operations)
            .and_then(|()| admission.validate_current_authority())
            .map_err(WorthQueryManagedApplicationRecoveryDenial::Authorization)?;
        self.with_application_advancement(admission.publication_request(), |phase| {
            self.recover_admitted_unpublished_application_in_advancement(
                &phase,
                recovery,
                admission,
                idempotency,
            )
        })
        .map_err(WorthQueryManagedApplicationRecoveryDenial::ExecutionDenied)?
    }

    /// Continues recovery inside the Publication call that already admitted its readers.
    pub fn recover_admitted_unpublished_application_in_advancement<Operation, Input, Scope>(
        &self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        recovery: &WorthQueryProductUnpublishedRecovery,
        admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> Result<
        WorthQueryManagedApplicationRecoveryOutcome,
        WorthQueryManagedApplicationRecoveryDenial,
    >
    where
        Input: Clone + Send + Sync + 'static,
    {
        self.recover_admitted_unpublished_application_delivery(
            phase,
            recovery,
            admission,
            idempotency,
            RecoveredPublicationDelivery::Ordinary,
        )
        .map(|(outcome, _)| outcome)
    }

    /// Retains the original performed output carrier before optional Query
    /// projection. Only the program-source owner uses this handoff.
    pub fn recover_admitted_unpublished_program_source<Operation, Input, Scope>(
        &self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        recovery: &WorthQueryProductUnpublishedRecovery,
        admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> Result<(WorthQueryManagedApplicationRecoveryOutcome, Option<crate::domain_computation::primary_graph::application_installation::WorthQueryRecoveredProgramOutputSource>), WorthQueryManagedApplicationRecoveryDenial>
    where Input: Clone + Send + Sync + 'static,
{
        self.recover_admitted_unpublished_application_delivery(
            phase,
            recovery,
            admission,
            idempotency,
            RecoveredPublicationDelivery::ProgramSource,
        )
    }

    /// Owning no-output recovery retains cleanup without constructing an output carrier.
    pub fn recover_admitted_unpublished_source_operation<Operation, Input, Scope>(
        &self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        recovery: &WorthQueryProductUnpublishedRecovery,
        admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> Result<
        WorthQueryManagedApplicationRecoveryOutcome,
        WorthQueryManagedApplicationRecoveryDenial,
    >
    where
        Input: Clone + Send + Sync + 'static,
    {
        self.recover_admitted_unpublished_application_delivery(
            phase,
            recovery,
            admission,
            idempotency,
            RecoveredPublicationDelivery::OwningSource,
        )
        .map(|(outcome, _)| outcome)
    }
}
#[cfg(test)]
#[path = "managed_application_recovery_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;
