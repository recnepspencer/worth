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
pub struct WorthQueryRetainedGovernedInput {
    material: Option<(
        worth_query_declaration::facade::application_operation::ApplicationCanonicalInput,
        &'static str,
        [u8; 32],
    )>,
    captured: Option<Arc<dyn Any + Send + Sync>>,
    _value: Arc<dyn Any + Send + Sync>,
    type_id: TypeId,
    type_name: &'static str,
    governed_identity: Option<[u8; 32]>,
    original_scope: Option<Arc<super::super::WorthQueryObservedScopeSelector>>,
    correction:
        Option<Arc<crate::domain_computation::application_aftermath::OriginalCorrectionDisclosure>>,
}

impl WorthQueryRetainedGovernedInput {
    #[expect(
        clippy::disallowed_methods,
        reason = "Type identity and names serve point lookup or diagnostics; canonical order uses declared identity"
    )]
    pub fn from_admission<Input>(
        input: &ApplicationSharedInput<Input>,
        governed_identity: Option<[u8; 32]>,
        original_scope: Option<&Arc<super::super::WorthQueryObservedScopeSelector>>,
        correction: Option<
            &Arc<crate::domain_computation::application_aftermath::OriginalCorrectionDisclosure>,
        >,
        captured: Option<&Arc<dyn Any + Send + Sync>>,
        material: Option<(
            &worth_query_declaration::facade::application_operation::ApplicationCanonicalInput,
            &'static str,
            [u8; 32],
        )>,
    ) -> Self
    where
        Input: Send + Sync + 'static,
    {
        Self {
            material: material
                .map(|(snapshot, binding, identity)| (snapshot.clone(), binding, identity)),
            captured: captured.cloned(),
            _value: Arc::new(input.clone()),
            type_id: TypeId::of::<Input>(),
            type_name: std::any::type_name::<Input>(),
            governed_identity,
            original_scope: original_scope.cloned(),
            correction: correction.cloned(),
        }
    }

    fn downcast_ref<Input: 'static>(&self) -> Option<&Input> {
        (self.type_id == TypeId::of::<Input>())
            .then(|| {
                self._value
                    .downcast_ref::<ApplicationSharedInput<Input>>()
                    .map(AsRef::as_ref)
            })
            .flatten()
    }

    pub fn material_input(
        &self,
    ) -> Option<(
        &worth_query_declaration::facade::application_operation::ApplicationCanonicalInput,
        &'static str,
        [u8; 32],
    )> {
        self.material
            .as_ref()
            .map(|(snapshot, binding, identity)| (snapshot, *binding, *identity))
    }
    pub fn retained_input<Binding>(
        &self,
    ) -> Option<
        &worth_query_declaration::facade::application_operation::ApplicationRetainedInput<Binding>,
    >
    where
        Binding:
            worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding,
    {
        self.captured.as_ref()?.downcast_ref()
    }

    pub const fn governed_identity(&self) -> Option<[u8; 32]> {
        self.governed_identity
    }

    pub fn original_scope(
        &self,
    ) -> Option<&Arc<super::super::WorthQueryObservedScopeSelector>> {
        self.original_scope.as_ref()
    }
    pub fn correction(
        &self,
    ) -> Option<&crate::domain_computation::application_aftermath::OriginalCorrectionDisclosure>
    {
        self.correction.as_deref()
    }

    #[expect(
        clippy::disallowed_methods,
        reason = "Type identity and names serve point lookup or diagnostics; canonical order uses declared identity"
    )]
    pub fn axis_probe(governed_identity: [u8; 32]) -> Self {
        Self {
            material: None,
            captured: None,
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
#[derive(Clone)]
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
    original_program: Option<Arc<dyn Any + Send + Sync>>,
    original_inspection_disclosure:
        Option<Arc<super::super::source_disclosure::OriginalSourceInspectionDisclosure>>,
}

impl std::fmt::Debug for WorthQueryApplicationCommitAuthorityBinding {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthQueryApplicationCommitAuthorityBinding")
            .field("original_mutation_binding", &self.original_mutation_binding)
            .field("runtime_authority", &self.runtime_authority)
            .field("installed_operation", &self.installed_operation)
            .field("installed_aftermath", &self.installed_aftermath)
            .field("principal_scope", &self.principal_scope)
            .field("idempotency_binding", &self.idempotency_binding)
            .field("retained_governed_input", &self.retained_governed_input)
            .field("has_original_program", &self.original_program.is_some())
            .field(
                "original_inspection_disclosure",
                &self.original_inspection_disclosure,
            )
            .finish()
    }
}

impl PartialEq for WorthQueryApplicationCommitAuthorityBinding {
    fn eq(&self, other: &Self) -> bool {
        self.original_mutation_binding == other.original_mutation_binding
            && self.runtime_authority == other.runtime_authority
            && self.installed_operation == other.installed_operation
            && self.installed_aftermath == other.installed_aftermath
            && self.principal_scope == other.principal_scope
            && self.idempotency_binding == other.idempotency_binding
            && self.retained_governed_input == other.retained_governed_input
            && match (&self.original_program, &other.original_program) {
                (None, None) => true,
                (Some(a), Some(b)) => std::ptr::addr_eq(Arc::as_ptr(a), Arc::as_ptr(b)),
                _ => false,
            }
            && self.original_inspection_disclosure == other.original_inspection_disclosure
    }
}

impl Eq for WorthQueryApplicationCommitAuthorityBinding {}

impl WorthQueryApplicationCommitAuthorityBinding {
    #[allow(clippy::too_many_arguments)]
    pub fn from_parts(
        original_mutation_binding: Option<std::any::TypeId>,
        runtime_authority: WorthQueryRuntimeAuthorityIdentity,
        installed_operation: [u8; 32],
        installed_aftermath: Option<WorthQueryInstalledAftermathContract>,
        principal_scope: WorthQueryOperationScopeBinding,
        idempotency_binding: WorthQueryApplicationIdempotencyBinding,
        retained_governed_input: Option<WorthQueryRetainedGovernedInput>,
        original_program: Option<Arc<dyn Any + Send + Sync>>,
        original_inspection_disclosure: Option<
            Arc<super::super::source_disclosure::OriginalSourceInspectionDisclosure>,
        >,
    ) -> Self {
        Self {
            original_mutation_binding,
            runtime_authority,
            installed_operation,
            installed_aftermath,
            principal_scope,
            idempotency_binding,
            retained_governed_input,
            original_program,
            original_inspection_disclosure,
        }
    }

    /// Query runtime authority that admitted this commit's operation.
    pub const fn runtime_authority(&self) -> WorthQueryRuntimeAuthorityIdentity {
        self.runtime_authority
    }

    pub const fn installed_operation(&self) -> &[u8; 32] {
        &self.installed_operation
    }

    pub const fn original_mutation_binding(&self) -> Option<std::any::TypeId> {
        self.original_mutation_binding
    }

    pub fn original_program_any(&self) -> Option<&Arc<dyn Any + Send + Sync>> {
        self.original_program.as_ref()
    }

    /// Exact compiled aftermath retained from the admitted installed operation.
    pub const fn installed_aftermath(
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
    pub fn retained_governed_input<Input: 'static>(&self) -> Option<&Input> {
        self.retained_governed_input.as_ref()?.downcast_ref()
    }

    pub const fn retained_governed_input_carrier(
        &self,
    ) -> Option<&WorthQueryRetainedGovernedInput> {
        self.retained_governed_input.as_ref()
    }

    pub fn original_inspection_disclosure(
        &self,
    ) -> Option<&super::super::source_disclosure::OriginalSourceInspectionDisclosure> {
        self.original_inspection_disclosure.as_deref()
    }

    pub fn retain_original_inspection_disclosure(
        &self,
    ) -> Option<Arc<super::super::source_disclosure::OriginalSourceInspectionDisclosure>> {
        self.original_inspection_disclosure.clone()
    }

    pub fn corrupt_original_disclosure(
        &mut self,
        disclosure: Arc<super::super::source_disclosure::OriginalSourceInspectionDisclosure>,
    ) {
        self.original_inspection_disclosure = Some(disclosure);
    }

    pub fn replace_idempotency_binding_for_test(
        &mut self,
        binding: WorthQueryApplicationIdempotencyBinding,
    ) {
        self.idempotency_binding = binding;
    }
}

mod court_doors_for_test;
