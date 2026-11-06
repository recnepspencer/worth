//! Complete source intent captured from a real plan, without its grant or session.
mod owned_scope;
mod source_read;
use super::output_read::{control_denial, HistoryOutputControlPlan};
use super::*;
use crate::domain_computation::primary_graph::{
    application_query::execution_validation::{
        validate_execution_lifetimes, validate_execution_plan,
    },
    product_operation::document_history::HistoryModelIdentity,
    WorthQueryHistoryNavigationDenial, WorthQueryHistoryOutputReadDenial,
    WorthQueryObservedScopeSelector,
};
use std::sync::Arc;
use worth_query_admission::facade::application_query::WorthQueryAdmittedApplicationQueryParameters;
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;

/// Immutable source correspondence. Fresh current permission is always separate.
#[derive(Clone)]
pub struct WorthQueryHistoryOutputIntent {
    backing: Arc<IntentBacking>,
}

#[derive(Debug)]
pub enum WorthQueryHistoryOutputNavigationDenial {
    Navigation(WorthQueryHistoryNavigationDenial),
    Source(WorthQueryHistoryOutputReadDenial),
}

struct IntentBacking {
    model: HistoryModelIdentity,
    binding: std::any::TypeId,
    query: worth_query_installation::facade::WorthQueryInstalledApplicationQueryIdentity,
    parameters: WorthQueryAdmittedApplicationQueryParameters,
    scope: Arc<WorthQueryObservedScopeSelector>,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
}

fn scope_work(scope: &WorthQueryObservedScopeSelector) -> Result<u64, Cause> {
    scope
        .value()
        .semantic_byte_width()
        .checked_add(scope.locator().owned_allocation_capacity_bytes())
        .and_then(|n| u64::try_from(n).ok())
        .ok_or(Cause::ChargedBytesOverflow)
}

fn fund_comparison(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    left: &WorthQueryAdmittedApplicationQueryParameters,
    right: &WorthQueryAdmittedApplicationQueryParameters,
    scope_work: u64,
) -> Result<Option<worth_execution::ExecutionMemoryReservation>, Cause> {
    let (left_bytes, left_work) = left
        .readmission_preparation_requirements()
        .ok_or(Cause::ChargedBytesOverflow)?;
    let (right_bytes, right_work) = right
        .readmission_preparation_requirements()
        .ok_or(Cause::ChargedBytesOverflow)?;
    let bytes = left_bytes
        .checked_add(right_bytes)
        .ok_or(Cause::ChargedBytesOverflow)?;
    let work = left_work
        .checked_add(right_work)
        .and_then(|n| n.checked_add(scope_work))
        .ok_or(Cause::ChargedBytesOverflow)?;
    let reservation = crate::domain_computation::primary_graph::structural_walk::memory::reserve(
        execution,
        execution.has_accounting(),
        bytes,
    )?;
    match execution.run_native_with_checkpoint(|context| {
        crate::domain_computation::primary_graph::admitted_capture::checkpoint(context, work)
    }) {
        Ok((result, _)) => result?,
        Err(stop) => return Err(stop.into()),
    }
    Ok(reservation)
}

impl WorthQueryHistoryOutputIntent {
    pub(in crate::domain_computation::primary_graph) fn source_subject(
        &self,
    ) -> worth_relational::facade::identity::EntityId {
        self.backing.scope.subject()
    }

    pub(in crate::domain_computation::primary_graph) fn source_partition(&self) -> [u8; 32] {
        *self.backing.parameters.identity().bytes()
    }

    pub(in crate::domain_computation::primary_graph) fn matches_output_source<Q>(
        &self,
        source: &crate::domain_computation::primary_graph::WorthQueryObservedSource<Q>,
        observation: &worth_runtime_world::facade::ProductBranchObservation,
    ) -> bool {
        self.backing.scope.matches_source(source)
            && matches!(&source.selection,
            crate::domain_computation::primary_graph::WorthQueryApplicationBasisSelectionIdentity::Product(product)
                if product.matches_observation(observation))
    }
    /// This identifies original immutable custody, not semantic equivalence.
    pub(in crate::domain_computation::primary_graph) fn same_custody(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.backing, &other.backing)
    }

    pub(in crate::domain_computation::primary_graph) fn equivalent(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        other: &Self,
    ) -> Result<bool, Cause> {
        if self.same_custody(other) {
            return Ok(true);
        }
        let _comparison = fund_comparison(
            execution,
            &self.backing.parameters,
            &other.backing.parameters,
            scope_work(&self.backing.scope)?
                .checked_add(scope_work(&other.backing.scope)?)
                .ok_or(Cause::ChargedBytesOverflow)?,
        )?;
        Ok(self.backing.model == other.backing.model
            && self.backing.binding == other.backing.binding
            && self.backing.query == other.backing.query
            && self.backing.scope.subject() == other.backing.scope.subject()
            && self.backing.scope.locator() == other.backing.scope.locator()
            && self.backing.scope.value() == other.backing.scope.value()
            && self
                .backing
                .parameters
                .canonical_basis()
                .is_equivalent_to(other.backing.parameters.canonical_basis()))
    }
    pub(in crate::domain_computation::primary_graph) fn matches_binding<
        S: ApplicationSchema,
        B: ApplicationQueryBinding<S>,
    >(
        &self,
        model: &HistoryModelIdentity,
        binding: &WorthQueryInstalledApplicationQueryBinding<S, B>,
    ) -> bool {
        &self.backing.model == model
            && self.backing.binding
                == crate::equality_type_id::EqualityTypeId::of::<B>().into_external()
            && &self.backing.query == binding.query().identity()
    }

    /// Compare the complete owner-issued canonical artifact, never a caller digest.
    pub(in crate::domain_computation::primary_graph) fn require_control<
        S: ApplicationSchema,
        B: ApplicationQueryBinding<S>,
    >(
        &self,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<S>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        binding: &WorthQueryInstalledApplicationQueryBinding<S, B>,
        control: &HistoryOutputControlPlan<'_, S, B>,
    ) -> Result<(), WorthQueryHistoryOutputReadDenial> {
        use WorthQueryHistoryOutputReadDenial as ReadDenial;
        validate_execution_plan(runtime, control).map_err(control_denial)?;
        validate_execution_lifetimes(runtime, &control.controls, control.principal)
            .map_err(control_denial)?;
        let _comparison = fund_comparison(
            execution,
            &self.backing.parameters,
            &control.parameters,
            scope_work(&self.backing.scope).map_err(ReadDenial::Preparation)?,
        )
        .map_err(ReadDenial::Preparation)?;
        let compared = execution.run_native_with_checkpoint(|_context| {
            Ok(self.matches_binding(&self.backing.model, binding)
                && control.query.identity() == binding.query().identity()
                && self.backing.scope.subject() == control.scope.entity_id()
                && self.backing.scope.locator() == control.scope.identity_locator()
                && self.backing.scope.value() == control.scope.identity_value()
                && self
                    .backing
                    .parameters
                    .canonical_basis()
                    .is_equivalent_to(control.parameters.canonical_basis()))
        });
        let matches = match compared {
            Ok((result, _)) => result.map_err(ReadDenial::Preparation)?,
            Err(stop) => return Err(ReadDenial::Preparation(stop.into())),
        };
        if !matches {
            return Err(ReadDenial::Authority(
                WorthQueryHistoryNavigationDenial::IntentDrift,
            ));
        }
        Ok(())
    }
}

impl<S: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<S> {
    pub fn capture_history_output_intent<
        H: ApplicationHistoryBinding<S>,
        B: ApplicationQueryBinding<S>,
    >(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        _: &WorthQueryProgramPublicationAccess,
        document: &WorthQueryDocumentHistory<'_, S, H>,
        binding: &WorthQueryInstalledApplicationQueryBinding<S, B>,
        control: HistoryOutputControlPlan<'_, S, B>,
    ) -> Result<WorthQueryHistoryOutputIntent, WorthQueryHistoryOutputReadDenial> {
        use WorthQueryHistoryOutputReadDenial as ReadDenial;
        validate_execution_plan(self, &control).map_err(control_denial)?;
        validate_execution_lifetimes(self, &control.controls, control.principal)
            .map_err(control_denial)?;
        if !std::ptr::eq(self, document.application())
            || control.query.identity() != binding.query().identity()
            || !control.controls.is_current_product_one_shot()
            || control.basis.selected_product().observation()
                != control.security_product.observation()
        {
            return Err(ReadDenial::Authority(
                WorthQueryHistoryNavigationDenial::ForeignDocument,
            ));
        }
        let execution = phase
            .checked_execution_request_for(&self.runtime.product_runtime)
            .map_err(|cause| {
                ReadDenial::Authority(WorthQueryHistoryNavigationDenial::Stopped(cause))
            })?;
        self.require_history_output_scope(
            &execution,
            document,
            control.scope,
            control.basis.selected_product(),
        )?;
        let (layout, _) = std::alloc::Layout::new::<[usize; 2]>()
            .extend(std::alloc::Layout::new::<IntentBacking>())
            .map_err(|_| ReadDenial::Preparation(Cause::ChargedBytesOverflow))?;
        let bytes = control
            .parameters
            .retained_owned_capacity_bytes()
            .and_then(|n| n.checked_add(layout.pad_to_align().size()))
            .and_then(|n| u64::try_from(n).ok())
            .ok_or(ReadDenial::Preparation(Cause::ChargedBytesOverflow))?;
        let memory = crate::domain_computation::primary_graph::structural_walk::memory::reserve(
            &execution,
            execution.has_accounting(),
            bytes,
        )
        .map_err(ReadDenial::Preparation)?;
        let scope = control
            .scope
            .retain_scope_selector_funded(&execution)
            .map_err(|cause| {
                use crate::domain_computation::primary_graph::{
                    WorthQueryApplicationQueryAdmissionDenial as Denial,
                    WorthQueryApplicationQueryAdmissionDenialKind as Kind,
                };
                use crate::domain_computation::WorthQueryProviderSessionControlStopKind as Stop;
                ReadDenial::Admission(Denial::new(
                    match cause {
                        Ok(cause) => Kind::ExecutionRequest(cause),
                        Err(Stop::Cancelled) => Kind::Cancelled,
                        Err(Stop::TimedOut) => Kind::DeadlineExceeded,
                    },
                    "history source intent capture",
                ))
            })?;
        Ok(WorthQueryHistoryOutputIntent {
            backing: Arc::new(IntentBacking {
                model: document.model.clone(),
                binding: crate::equality_type_id::EqualityTypeId::of::<B>().into_external(),
                query: binding.query().identity().clone(),
                parameters: control.parameters,
                scope,
                _memory: memory,
            }),
        })
    }
}
