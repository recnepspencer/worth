//! Authority fields a committed receipt must carry (R8.62 / C1).
//!
//! These are derived from the admitted operation and the attempt's idempotency
//! binding. Callers cannot supply them as assertions.

use crate::equality_type_id::EqualityTypeId as TypeId;
use std::any::Any;
use std::sync::Arc;

use crate::domain_computation::authorization::WorthQueryOperationScopeBinding;
use crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity;
use worth_query_installation::facade::WorthQueryInstalledAftermathContract;

use super::WorthQueryApplicationIdempotencyBinding;
use worth_query_declaration::facade::application_operation::ApplicationSharedInput;

/// Exact capability input retained from the original operation admission.
///
/// The erased storage lets the non-generic commit receipt carry application
/// input and its funded allocation through receipt and recovery custody.
/// Correction preparation accesses the original binding's encoded carrier.
#[derive(Clone)]
pub(crate) struct WorthQueryRetainedGovernedInput {
    encoded: Option<Arc<dyn Any + Send + Sync>>,
    _value: Arc<dyn Any + Send + Sync>,
    type_id: TypeId,
    type_name: &'static str,
    governed_identity: Option<[u8; 32]>,
    original_scope: Option<Arc<super::super::WorthQueryObservedScopeSelector>>,
    correction:
        Option<Arc<crate::domain_computation::application_aftermath::OriginalCorrectionDisclosure>>,
}

impl WorthQueryRetainedGovernedInput {
    fn from_admission<Input>(
        input: &ApplicationSharedInput<Input>,
        governed_identity: Option<[u8; 32]>,
        original_scope: Option<&Arc<super::super::WorthQueryObservedScopeSelector>>,
        correction: Option<
            &Arc<crate::domain_computation::application_aftermath::OriginalCorrectionDisclosure>,
        >,
        encoded: Option<&Arc<dyn Any + Send + Sync>>,
    ) -> Self
    where
        Input: Send + Sync + 'static,
    {
        Self {
            encoded: encoded.cloned(),
            _value: Arc::new(input.clone()),
            type_id: TypeId::of::<Input>(),
            type_name: std::any::type_name::<Input>(),
            governed_identity,
            original_scope: original_scope.cloned(),
            correction: correction.cloned(),
        }
    }

    #[cfg(test)]
    fn downcast_ref<Input: 'static>(&self) -> Option<&Input> {
        (self.type_id == TypeId::of::<Input>())
            .then(|| {
                self._value
                    .downcast_ref::<ApplicationSharedInput<Input>>()
                    .map(AsRef::as_ref)
            })
            .flatten()
    }

    pub(in crate::domain_computation::primary_graph) fn encoded_input<Binding>(
        &self,
    ) -> Option<
        &worth_query_declaration::facade::application_operation::ApplicationEncodedInput<Binding>,
    >
    where
        Binding:
            worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding,
    {
        self.encoded.as_ref()?.downcast_ref()
    }

    pub(crate) const fn governed_identity(&self) -> Option<[u8; 32]> {
        self.governed_identity
    }

    pub(in crate::domain_computation) fn original_scope(
        &self,
    ) -> Option<&Arc<super::super::WorthQueryObservedScopeSelector>> {
        self.original_scope.as_ref()
    }
    pub(in crate::domain_computation) fn correction(
        &self,
    ) -> Option<&crate::domain_computation::application_aftermath::OriginalCorrectionDisclosure>
    {
        self.correction.as_deref()
    }

    #[cfg(test)]
    pub(crate) fn axis_probe(governed_identity: [u8; 32]) -> Self {
        Self {
            encoded: None,
            _value: Arc::new(()),
            type_id: TypeId::of::<()>(),
            type_name: std::any::type_name::<()>(),
            governed_identity: Some(governed_identity),
            original_scope: None,
            correction: None,
        }
    }
}

impl std::fmt::Debug for WorthQueryRetainedGovernedInput {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthQueryRetainedGovernedInput")
            .field("type_name", &self.type_name)
            .field("governed_identity", &self.governed_identity)
            .finish_non_exhaustive()
    }
}

impl PartialEq for WorthQueryRetainedGovernedInput {
    fn eq(&self, other: &Self) -> bool {
        self.type_id == other.type_id && self.governed_identity == other.governed_identity
    }
}

impl Eq for WorthQueryRetainedGovernedInput {}

/// Installed operation, admitted principal scope, and idempotency binding
/// retained on a commit receipt so recovery can bind to admitted truth.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryApplicationCommitAuthorityBinding {
    original_mutation_binding: Option<std::any::TypeId>,
    /// Query runtime that admitted the operation this receipt commits.
    ///
    /// Not the Relational `provider_runtime_instance_id`, which identifies the
    /// relational instance and is shared by every Query runtime published over
    /// the same source. Recovery mint compares *this* against the runtime it is
    /// minting into, so one runtime cannot mint a handle for another's commit.
    runtime_authority: WorthQueryRuntimeAuthorityIdentity,
    installed_operation: [u8; 32],
    installed_aftermath: Option<WorthQueryInstalledAftermathContract>,
    principal_scope: WorthQueryOperationScopeBinding,
    idempotency_binding: WorthQueryApplicationIdempotencyBinding,
    retained_governed_input: Option<WorthQueryRetainedGovernedInput>,
    original_program: Option<Arc<super::super::OriginalProgramMeaning>>,
    original_inspection_disclosure:
        Option<Arc<super::super::source_disclosure::OriginalSourceInspectionDisclosure>>,
}

impl WorthQueryApplicationCommitAuthorityBinding {
    /// Derive the binding from what admission actually authorized.
    pub(in crate::domain_computation::primary_graph) fn from_admission<
        Schema,
        Operation,
        Input,
        Scope,
    >(
        admission: &crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationOperation<
            Schema,
            Operation,
            Input,
            Scope,
        >,
        idempotency_binding: WorthQueryApplicationIdempotencyBinding,
    ) -> Self
    where
        Input: Clone + Send + Sync + 'static,
    {
        let retained_governed_input = match admission.original_input() {
            Some((input, identity)) => Some(WorthQueryRetainedGovernedInput::from_admission(
                input,
                Some(identity),
                admission.original_scope(),
                admission.original_correction_disclosure(),
                admission.original_encoded_input(),
            )),
            None => admission.shared_capability_input().map(|input| {
                WorthQueryRetainedGovernedInput::from_admission(
                    input,
                    admission.governed_input_identity().copied(),
                    admission.original_scope(),
                    admission.original_correction_disclosure(),
                    admission.original_encoded_input(),
                )
            }),
        };
        Self {
            original_mutation_binding: admission.original_mutation_binding(),
            runtime_authority: admission.runtime_authority(),
            installed_operation: *admission.installed_operation_identity_bytes(),
            installed_aftermath: admission.allowed_graph_contract().aftermath().cloned(),
            principal_scope: admission.operation_scope_binding().clone(),
            idempotency_binding,
            retained_governed_input,
            original_program: admission.original_program().cloned(),
            original_inspection_disclosure: admission.original_inspection_disclosure().cloned(),
        }
    }

    /// Query runtime authority that admitted this commit's operation.
    pub(crate) const fn runtime_authority(&self) -> WorthQueryRuntimeAuthorityIdentity {
        self.runtime_authority
    }

    pub const fn installed_operation(&self) -> &[u8; 32] {
        &self.installed_operation
    }

    pub(crate) const fn original_mutation_binding(&self) -> Option<std::any::TypeId> {
        self.original_mutation_binding
    }

    pub(in crate::domain_computation::primary_graph) fn original_program(
        &self,
    ) -> Option<&super::super::OriginalProgramMeaning> {
        self.original_program.as_deref()
    }

    /// Exact compiled aftermath retained from the admitted installed operation.
    pub(crate) const fn installed_aftermath(
        &self,
    ) -> Option<&WorthQueryInstalledAftermathContract> {
        self.installed_aftermath.as_ref()
    }

    pub const fn principal_scope(&self) -> &WorthQueryOperationScopeBinding {
        &self.principal_scope
    }

    pub const fn idempotency_binding(&self) -> WorthQueryApplicationIdempotencyBinding {
        self.idempotency_binding
    }

    /// Recover the exact original admission input by its application type.
    ///
    /// A mismatched type yields no value; the caller never supplies replacement
    /// bytes or a fallback value.
    #[cfg(test)]
    pub(crate) fn retained_governed_input<Input: 'static>(&self) -> Option<&Input> {
        self.retained_governed_input.as_ref()?.downcast_ref()
    }

    pub(crate) const fn retained_governed_input_carrier(
        &self,
    ) -> Option<&WorthQueryRetainedGovernedInput> {
        self.retained_governed_input.as_ref()
    }

    pub(in crate::domain_computation::primary_graph) fn original_inspection_disclosure(
        &self,
    ) -> Option<&super::super::source_disclosure::OriginalSourceInspectionDisclosure> {
        self.original_inspection_disclosure.as_deref()
    }

    #[cfg(test)]
    pub(crate) fn replace_idempotency_binding_for_test(
        &mut self,
        binding: WorthQueryApplicationIdempotencyBinding,
    ) {
        self.idempotency_binding = binding;
    }
}
