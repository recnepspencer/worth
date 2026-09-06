use crate::branch::ProductBranchObservation;
use crate::lifecycle::RuntimeWorldInstant;
use crate::publication::{
    lower_component_plans, LoweredOwnerComponentPlan, NoEffectCause, NoEffectCompositePublication,
    ReservedAttemptCapacities, ReservedAttemptCapacityInputs, ReservedCompositePublicationAttempt,
    ResolvedExpectedProductHead, RuntimeWorldCancellationBoundary, RuntimeWorldCancellationToken,
};

use super::RuntimeWorldOwnerRoot;

mod attempt;
#[path = "operation/creation_reservation.rs"]
mod creation_reservation;
mod publication_capacity;
mod reservation_steps;

#[cfg(test)]
#[path = "operation/preparation_tests.rs"]
mod preparation_tests;

#[cfg(test)]
#[path = "operation/preparation_test_support.rs"]
mod preparation_test_support;

pub(crate) use attempt::{
    RuntimeWorldOperationLedger, RuntimeWorldOperationReservation, RuntimeWorldOperationState,
};
pub(crate) use publication_capacity::{
    ReservedPublicationAttemptCapacity, RuntimeWorldPublicationCapacityLedger,
};
use reservation_steps::{
    assemble_reserved_attempt, issue_publication_identities, reserve_publication_resources,
    ReservedAttemptAssembly,
};

struct ReservationContext<'a> {
    plan: LoweredOwnerComponentPlan,
    expected_head: ProductBranchObservation,
    cancellation: &'a RuntimeWorldCancellationToken,
    deadline: Option<RuntimeWorldInstant>,
    operation: RuntimeWorldOperationReservation,
}

impl<D, I, E, Ctx, T> RuntimeWorldOwnerRoot<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    /// The owner-only preparation transition. Identities and all bounded
    /// reservations are issued here; a lowered plan cannot reserve itself.
    pub(crate) fn reserve(
        &self,
        plan: LoweredOwnerComponentPlan,
        cancellation: &RuntimeWorldCancellationToken,
        deadline: Option<RuntimeWorldInstant>,
    ) -> Result<ReservedCompositePublicationAttempt, NoEffectCompositePublication> {
        let expected_head = plan.expected().expected().clone();
        self.validate_reservation_preconditions(&plan, &expected_head, cancellation, deadline)
            .map_err(|cause| {
                NoEffectCompositePublication::new(cause, Some(expected_head.clone()))
            })?;
        let operation = match self.reserve_operation_if_open_and_bootstrapped() {
            Ok(operation) => operation,
            Err(()) => {
                return Err(NoEffectCompositePublication::new(
                    NoEffectCause::OwnerUnavailable,
                    Some(expected_head),
                ))
            }
        };
        self.complete_reservation(ReservationContext {
            plan,
            expected_head,
            cancellation,
            deadline,
            operation,
        })
    }

    fn complete_reservation(
        &self,
        context: ReservationContext<'_>,
    ) -> Result<ReservedCompositePublicationAttempt, NoEffectCompositePublication> {
        let ReservationContext {
            plan,
            expected_head,
            cancellation,
            deadline,
            operation,
        } = context;
        if let Some(denied) = self.reservation_denial(&expected_head, cancellation, deadline) {
            return Err(denied);
        }
        let identities = issue_publication_identities(self).map_err(|cause| {
            NoEffectCompositePublication::new(cause, Some(expected_head.clone()))
        })?;
        if let Some(denied) = self.reservation_denial(&expected_head, cancellation, deadline) {
            return Err(denied);
        }
        let resources = reserve_publication_resources(
            self,
            &expected_head,
            &identities.commit_identity,
            Some(&identities.attempt_identity),
        )
        .map_err(|cause| NoEffectCompositePublication::new(cause, Some(expected_head.clone())))?;
        if let Some(denied) = self.reservation_denial(&expected_head, cancellation, deadline) {
            return Err(denied);
        }
        if !self.current_product_head_is(&expected_head) {
            return Err(NoEffectCompositePublication::new(
                NoEffectCause::StaleExpectedProductHead,
                Some(expected_head.clone()),
            ));
        }
        Ok(assemble_reserved_attempt(ReservedAttemptAssembly {
            admitted_at: self.state.clock.now(),
            identities,
            resources,
            plan,
            expected_head,
            deadline,
            operation,
        }))
    }

    fn validate_reservation_preconditions(
        &self,
        plan: &LoweredOwnerComponentPlan,
        expected: &crate::branch::ProductBranchObservation,
        cancellation: &RuntimeWorldCancellationToken,
        deadline: Option<RuntimeWorldInstant>,
    ) -> Result<(), NoEffectCause> {
        if let Some(cause) = self.pre_effect_denial(cancellation, deadline) {
            return Err(cause);
        }
        if expected.owner_identity() != self.owner_identity() {
            return Err(NoEffectCause::OwnerDeniedBeforeEffect);
        }
        if !plan.is_internally_consistent() {
            return Err(NoEffectCause::PreEffectFailure);
        }
        if !self.current_product_head_is(expected) {
            return Err(NoEffectCause::StaleExpectedProductHead);
        }
        if expected.reference_generation().advance().is_err() {
            return Err(NoEffectCause::ReferenceGenerationExhausted);
        }
        Ok(())
    }

    fn reservation_denial(
        &self,
        expected: &crate::branch::ProductBranchObservation,
        cancellation: &RuntimeWorldCancellationToken,
        deadline: Option<RuntimeWorldInstant>,
    ) -> Option<NoEffectCompositePublication> {
        self.pre_effect_denial(cancellation, deadline)
            .map(|cause| NoEffectCompositePublication::new(cause, Some(expected.clone())))
    }

    pub(super) fn current_product_head_is(
        &self,
        expected: &crate::branch::ProductBranchObservation,
    ) -> bool {
        self.state
            .branches
            .branch_cell(expected.branch_identity())
            .map(|cell| cell.atomic_snapshot())
            .is_some_and(|current| expected.mismatch_against_snapshot(&current).is_none())
    }

    pub(super) fn current_product_head_snapshot(
        &self,
        expected: &crate::branch::ProductBranchObservation,
    ) -> Option<crate::branch::ProductBranchReferenceSnapshot> {
        self.state
            .branches
            .branch_cell(expected.branch_identity())
            .map(|cell| cell.atomic_snapshot())
    }

    fn pre_effect_denial(
        &self,
        cancellation: &RuntimeWorldCancellationToken,
        deadline: Option<RuntimeWorldInstant>,
    ) -> Option<NoEffectCause> {
        if cancellation
            .check(RuntimeWorldCancellationBoundary::BeforeReservation)
            .is_err()
        {
            return Some(NoEffectCause::CancelledBeforeEffect);
        }
        deadline
            .filter(|deadline| self.state.clock.now() >= *deadline)
            .map(|_| NoEffectCause::DeadlineBeforeEffect)
    }

    fn reserve_operation_if_open_and_bootstrapped(
        &self,
    ) -> Result<RuntimeWorldOperationReservation, ()> {
        self.reserve_operation_with_state(RuntimeWorldOperationState::Preparing)
    }

    pub(in crate::lifecycle) fn reserve_recovery_operation_if_open_and_bootstrapped(
        &self,
    ) -> Result<RuntimeWorldOperationReservation, ()> {
        self.reserve_operation_with_state(RuntimeWorldOperationState::Recovering)
    }

    fn reserve_operation_with_state(
        &self,
        initial_state: RuntimeWorldOperationState,
    ) -> Result<RuntimeWorldOperationReservation, ()> {
        if !matches!(
            initial_state,
            RuntimeWorldOperationState::Preparing | RuntimeWorldOperationState::Recovering
        ) {
            return Err(());
        }
        let bootstrap = self
            .state
            .bootstrap
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        if *bootstrap != super::RuntimeWorldBootstrapState::Performed {
            return Err(());
        }
        let mut ledger = self
            .state
            .operation
            .state
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let close = self
            .state
            .close
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        if !self.owner_is_present()
            || close.state() != super::super::close::RuntimeWorldCloseState::Open
        {
            return Err(());
        }
        let active = ledger.active.checked_add(1).ok_or(())?;
        let recovery_active = if initial_state == RuntimeWorldOperationState::Recovering {
            Some(ledger.recovery_active.checked_add(1).ok_or(())?)
        } else {
            None
        };
        ledger.active = active;
        if let Some(recovery_active) = recovery_active {
            ledger.recovery_active = recovery_active;
        }
        drop(close);
        drop(ledger);
        drop(bootstrap);
        Ok(if initial_state == RuntimeWorldOperationState::Recovering {
            self.state.operation.recovering_reservation()
        } else {
            self.state.operation.preparing_reservation()
        })
    }

    fn is_open_and_bootstrapped(&self) -> bool {
        if !self.owner_is_present() {
            return false;
        }
        let bootstrap = self
            .state
            .bootstrap
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        if *bootstrap != super::RuntimeWorldBootstrapState::Performed {
            return false;
        }
        self.state
            .close
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .state()
            == super::super::close::RuntimeWorldCloseState::Open
    }
}

impl<D, I, E, Ctx, T> super::super::ports::RuntimeWorldPreparationService
    for RuntimeWorldOwnerRoot<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    fn prepare_publication<S>(
        &self,
        expected: ProductBranchObservation,
        intent: crate::publication::CompositePublicationIntent<S>,
        cancellation: &RuntimeWorldCancellationToken,
        deadline: Option<RuntimeWorldInstant>,
    ) -> Result<S::Prepared, NoEffectCompositePublication>
    where
        S: crate::publication::CompositePublicationStage,
    {
        let current = self.admit_publication_source(&expected)?;
        let (component_intent, prepared_candidate) = intent.into_parts();
        let resolved = match ResolvedExpectedProductHead::from_current(
            component_intent,
            expected.clone(),
            &current,
        ) {
            Ok(resolved) => resolved,
            Err(_) => {
                drop(prepared_candidate);
                return Err(NoEffectCompositePublication::new(
                    NoEffectCause::StaleExpectedProductHead,
                    Some(expected),
                ));
            }
        };
        let plan = lower_component_plans(resolved, prepared_candidate)?;
        let attempt = RuntimeWorldOwnerRoot::reserve(self, plan, cancellation, deadline)?;
        Ok(S::seal(attempt))
    }

    fn prepare_creation(
        &self,
        source: ProductBranchObservation,
        intent: crate::branch::ProductBranchCreationIntent,
        cancellation: &RuntimeWorldCancellationToken,
        deadline: Option<RuntimeWorldInstant>,
    ) -> Result<
        crate::publication::ReservedBranchCreationAttempt,
        crate::branch::RuntimeWorldBranchAdmissionDenial,
    > {
        RuntimeWorldOwnerRoot::prepare_creation(self, source, intent, cancellation, deadline)
    }
}

impl<D, I, E, Ctx, T> RuntimeWorldOwnerRoot<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    /// The shared pre-lowering admission: this owner, an open world, and a
    /// branch cell that still exists. It returns the exact current snapshot
    /// the caller must compare its expected observation against.
    fn admit_publication_source(
        &self,
        expected: &ProductBranchObservation,
    ) -> Result<crate::branch::ProductBranchReferenceSnapshot, NoEffectCompositePublication> {
        if expected.owner_identity() != self.owner_identity() {
            return Err(NoEffectCompositePublication::new(
                NoEffectCause::OwnerDeniedBeforeEffect,
                Some(expected.clone()),
            ));
        }
        if !self.is_open_and_bootstrapped() {
            return Err(NoEffectCompositePublication::new(
                NoEffectCause::OwnerUnavailable,
                Some(expected.clone()),
            ));
        }
        self.state
            .branches
            .branch_cell(expected.branch_identity())
            .map(|cell| cell.atomic_snapshot())
            .ok_or_else(|| {
                NoEffectCompositePublication::new(
                    NoEffectCause::OwnerUnavailable,
                    Some(expected.clone()),
                )
            })
    }

    pub(super) fn is_open_and_bootstrapped_for_creation(&self) -> bool {
        self.is_open_and_bootstrapped()
    }

    pub(super) fn reserve_creation_operation(
        &self,
    ) -> Result<RuntimeWorldOperationReservation, ()> {
        self.reserve_operation_if_open_and_bootstrapped()
    }
}
