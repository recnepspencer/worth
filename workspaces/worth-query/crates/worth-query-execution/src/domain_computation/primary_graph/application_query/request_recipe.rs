//! Immutable normalized request data; principals and grants are never captured.
use super::CapturedApplicationQueryParameters;
use crate::domain_computation::primary_graph::WorthQueryHistoryOutputReadDenial as Denial;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;
use sha2::{Digest, Sha256};
use std::sync::Arc;
use worth_execution::ExecutionRequest;
use worth_foundational::facade::{
    write_aspect_value_identity_material, AspectValue, CanonicalMaterialSink,
};
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
    binding: std::any::TypeId,
    parameters: CapturedApplicationQueryParameters,
    scope: Option<AspectValue>,
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
        let memory = crate::domain_computation::primary_graph::structural_walk::memory::reserve(
            execution,
            execution.has_accounting(),
            u64::try_from(bytes).map_err(|_| Denial::Preparation(Cause::ChargedBytesOverflow))?,
        )
        .map_err(Denial::Preparation)?;
        let mut captured = Self {
            binding: std::any::TypeId::of::<B>(),
            parameters,
            scope,
            field,
            seal: [0; 32],
            _memory: memory,
        };
        captured.seal = captured
            .content_digest(execution)
            .map_err(Denial::Preparation)?;
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
        self.scope.as_ref()
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
        self.binding == std::any::TypeId::of::<B>()
            && self.field == (field.entity(), field.aspect(), field.field())
    }
    pub(in crate::domain_computation::primary_graph) fn validate(
        &self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<bool, Cause> {
        self.content_digest(execution)
            .map(|digest| digest == self.seal)
    }
    fn content_digest(&self, execution: &ExecutionRequest<'_, '_>) -> Result<[u8; 32], Cause> {
        execution
            .run_native_with_checkpoint(|context| {
                let mut writer = RequestWriter {
                    digest: Sha256::new(),
                    context,
                };
                writer.bytes(self.parameters.query().as_bytes())?;
                for name in [self.field.0, self.field.1, self.field.2] {
                    writer.bytes(&(name.len() as u64).to_be_bytes())?;
                    writer.bytes(name.as_bytes())?;
                }
                writer.bytes(self.parameters.parameters().identity().bytes())?;
                writer.bytes(&[u8::from(self.scope.is_some())])?;
                if let Some(scope) = &self.scope {
                    write_aspect_value_identity_material(scope, &mut writer)?;
                }
                for (name, value) in self.parameters.parameters().bindings() {
                    writer.bytes(&(name.len() as u64).to_be_bytes())?;
                    writer.bytes(name.as_bytes())?;
                    write_aspect_value_identity_material(value, &mut writer)?;
                }
                Ok(writer.digest.finalize().into())
            })
            .map_err(Cause::from)?
            .0
    }
}
struct RequestWriter<'a, 'b, 'c> {
    digest: Sha256,
    context: &'a mut worth_execution::MapKernelContext<'b, 'c>,
}
impl RequestWriter<'_, '_, '_> {
    fn bytes(&mut self, bytes: &[u8]) -> Result<(), Cause> {
        for chunk in bytes.chunks(4096) {
            crate::domain_computation::primary_graph::admitted_capture::checkpoint(
                self.context,
                chunk.len() as u64,
            )?;
            self.digest.update(chunk);
        }
        Ok(())
    }
}
impl CanonicalMaterialSink for RequestWriter<'_, '_, '_> {
    type Error = Cause;
    fn append(&mut self, value: &str) -> Result<(), Cause> {
        self.bytes(value.as_bytes())
    }
    fn admit_work(&mut self, work: usize) -> Result<(), Cause> {
        crate::domain_computation::primary_graph::admitted_capture::checkpoint(
            self.context,
            work as u64,
        )
        .map_err(Cause::from)
    }
    fn accounting_overflow(&mut self) -> Cause {
        Cause::ChargedBytesOverflow
    }
}
fn invalid(
    step: crate::domain_computation::primary_graph::WorthQueryHistoryNodeId,
    binding: &'static str,
) -> Denial {
    Denial::Authority(crate::domain_computation::primary_graph::WorthQueryHistoryNavigationDenial::DiscoveryRecipeInvalidScope {step, binding})
}
