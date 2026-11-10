//! Immutable normalized request data; principals and grants are never captured.
use super::CapturedApplicationQueryParameters;
use crate::domain_computation::primary_graph::WorthQueryHistoryOutputReadDenial as Denial;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;
use crate::equality_type_id::EqualityTypeId;
use std::sync::Arc;
use worth_execution::ExecutionRequest;
use worth_foundational::facade::AspectValue;
use worth_query_admission::facade::application_query::WorthQueryAdmittedApplicationQueryParameters;
use worth_query_declaration::facade::{
    application_query::{
        ApplicationQueryBinding, ApplicationQueryIntent, ApplicationQueryScopeBinding,
        ApplicationQueryScopeResolution,
    },
    application_schema::{
        fragment::RecordSelection, ApplicationScalarValueBinding, ApplicationSchema,
    },
};
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;

type Field<S, B> =
    <<B as ApplicationQueryBinding<S>>::ScopeBinding as ApplicationQueryScopeBinding<S>>::Field;
type Scalar<S, B> = <Field<S, B> as RecordSelection>::Binding;

pub(in crate::domain_computation::primary_graph) struct CapturedSourceQueryRequest {
    binding: EqualityTypeId,
    parameters: CapturedApplicationQueryParameters,
    request: worth_query_admission::facade::application_query::WorthQueryCanonicalSourceRequest,
    field: (&'static str, &'static str, &'static str),
    seal: [u8; 32],
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
}
impl CapturedSourceQueryRequest {
    pub(in crate::domain_computation::primary_graph) fn capture<S, B>(
        execution: &ExecutionRequest<'_, '_>,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<S>,
        step: crate::domain_computation::primary_graph::WorthQueryHistoryNodeId,
        intent: B::Input,
    ) -> Result<Arc<Self>, Denial>
    where
        S: ApplicationSchema,
        B: ApplicationQueryBinding<S>,
        B::Input: ApplicationQueryIntent<S, Binding = B>,
        B::ScopeBinding: ApplicationQueryScopeResolution<S, B::PrincipalIdentity>,
    {
        let parameters = super::capture_parameters::<S, B>(
            execution,
            runtime,
            std::mem::size_of::<Self>() + 2 * std::mem::size_of::<usize>(),
            || Ok(intent.parameters()),
        )
        .map_err(Denial::RecipeCapture)?;
        let (field, value) = intent
            .into_scope()
            .into_request_parts()
            .ok_or(Denial::Authority(crate::domain_computation::primary_graph::WorthQueryHistoryNavigationDenial::DiscoveryRecipeUnsupportedScope {step, binding: B::IDENTITY}))?;
        let field = (field.entity(), field.aspect(), field.field());
        let expected = B::scope_field();
        if field != (expected.entity(), expected.aspect(), expected.field())
            || value.is_none() != matches!(<B::ScopeBinding as ApplicationQueryScopeBinding<S>>::RESOLUTION_MODE,
                worth_query_declaration::facade::application_query::ApplicationQueryScopeResolutionMode::PrincipalIdentity) {
            return Err(invalid(step, B::IDENTITY));
        }
        let scope = value
            .as_ref()
            .map(|value| {
                Scalar::<S, B>::validate(value).map_err(|_| invalid(step, B::IDENTITY))?;
                Scalar::<S, B>::encode(value).map_err(|_| invalid(step, B::IDENTITY))
            })
            .transpose()?;
        let bytes = scope
            .as_ref()
            .map_or(0, AspectValue::owned_allocation_capacity_bytes);
        let mut memory =
            crate::domain_computation::primary_graph::structural_walk::memory::reserve(
                execution,
                execution.has_accounting(),
                u64::try_from(bytes)
                    .map_err(|_| Denial::Preparation(Cause::ChargedBytesOverflow))?,
            )
            .map_err(Denial::Preparation)?;
        let installed = runtime
            .installed_schema()
            .installed_query_binding::<B>()
            .map_err(|_| invalid(step, B::IDENTITY))?;
        let mut retained =
            u64::try_from(bytes).map_err(|_| Denial::Preparation(Cause::ChargedBytesOverflow))?;
        let request = execution.run_native_with_checkpoint(|context| {
            parameters.parameters().capture_source_request_admitted(installed.query(), field, scope, |work, bytes| {
                crate::domain_computation::primary_graph::admitted_capture::checkpoint(context,
                    u64::try_from(work).map_err(|_| Cause::WorkCounterOverflow)?)?;
                retained = retained.checked_add(u64::try_from(bytes).map_err(|_| Cause::ChargedBytesOverflow)?)
                    .ok_or(Cause::ChargedBytesOverflow)?;
                crate::domain_computation::primary_graph::structural_walk::memory::resize(&mut memory, retained)
            })
        }).map_err(|cause| Denial::Preparation(cause.into()))?.0.map_err(|cause| {
            use worth_query_admission::facade::application_query::WorthQueryParameterPreparationStop as Stop;
            match cause {
                Stop::Admission(cause) | Stop::Preparation { cause, .. } => Denial::Preparation(cause),
                Stop::Parameters(cause) => Denial::RecipeCapture(
                    crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial::source_disclosure_parameters(cause)),
                Stop::AccountingOverflow => Denial::Preparation(Cause::ChargedBytesOverflow),
            }
        })?;
        let captured = Self {
            binding: EqualityTypeId::of::<B>(),
            parameters,
            field,
            seal: *request.identity().bytes(),
            request,
            _memory: memory,
        };
        Ok(Arc::new(captured))
    }
    #[doc(hidden)]
    pub(in crate::domain_computation::primary_graph) fn corrupt_seal_for_test(&mut self) {
        self.seal[0] ^= 1;
    }
    pub(in crate::domain_computation::primary_graph) fn parameters(
        &self,
    ) -> &WorthQueryAdmittedApplicationQueryParameters {
        self.parameters.parameters()
    }
    pub(in crate::domain_computation::primary_graph) fn scope(&self) -> Option<&AspectValue> {
        self.request.scope()
    }
    pub(in crate::domain_computation::primary_graph) fn matches_binding<
        S: ApplicationSchema,
        B: ApplicationQueryBinding<S>,
    >(
        &self,
        _installed: &worth_query_installation::facade::WorthQueryInstalledApplicationQueryBinding<
            S,
            B,
        >,
    ) -> bool {
        let field = B::scope_field();
        // The recorded query identity protects the original input. Current
        // installation meaning and parameter readmission remain the reader's
        // owner, including genuine changed-build fallback.
        self.binding == EqualityTypeId::of::<B>()
            && self.field == (field.entity(), field.aspect(), field.field())
    }
    pub(in crate::domain_computation::primary_graph) fn validate(
        &self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<bool, Cause> {
        let work = self
            .field
            .0
            .len()
            .checked_add(self.field.1.len())
            .and_then(|n| n.checked_add(self.field.2.len()))
            .and_then(|n| n.checked_add(96))
            .and_then(|n| u64::try_from(n).ok())
            .ok_or(Cause::WorkCounterOverflow)?;
        execution
            .run_native_with_checkpoint(|context| {
                crate::domain_computation::primary_graph::admitted_capture::checkpoint(
                    context, work,
                )
                .map_err(Cause::from)?;
                Ok(self.seal == *self.request.identity().bytes()
                    && self.request.matches(
                        self.parameters.query(),
                        self.parameters.parameters(),
                        self.field,
                    ))
            })
            .map_err(Cause::from)?
            .0
    }
}

fn invalid(
    step: crate::domain_computation::primary_graph::WorthQueryHistoryNodeId,
    binding: &'static str,
) -> Denial {
    Denial::Authority(crate::domain_computation::primary_graph::WorthQueryHistoryNavigationDenial::DiscoveryRecipeInvalidScope {step, binding})
}
