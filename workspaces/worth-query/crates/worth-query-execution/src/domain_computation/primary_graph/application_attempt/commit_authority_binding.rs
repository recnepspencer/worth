//! Authority fields a committed receipt must carry (R8.62 / C1).
//!
//! These are derived from the admitted operation and the attempt's idempotency
//! binding. Callers cannot supply them as assertions.

use std::sync::Arc;

use crate::domain_computation::authorization::WorthQueryOperationScopeBinding;
use crate::domain_computation::vocabulary::execution_runtime::runtime_identity::WorthQueryRuntimeAuthorityIdentity;
use worth_query_installation::facade::WorthQueryInstalledAftermathContract;

use crate::domain_computation::primary_graph::vocabulary::application_attempt::idempotency::WorthQueryApplicationIdempotencyBinding;
use crate::domain_computation::vocabulary::primary_graph::commit_authority_binding::WorthQueryRetainedGovernedInput;

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
        Option<Arc<crate::domain_computation::vocabulary::primary_graph::inspection::OriginalSourceInspectionDisclosure>>,
}

impl WorthQueryApplicationCommitAuthorityBinding {
    /// Derive the binding from what admission actually authorized.
    pub fn from_admission<Schema, Operation, Input, Scope>(
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
    ) -> Option<&crate::domain_computation::vocabulary::primary_graph::inspection::OriginalSourceInspectionDisclosure>{
        self.original_inspection_disclosure.as_deref()
    }

    pub(in crate::domain_computation::primary_graph) fn retain_original_inspection_disclosure(
        &self,
    ) -> Option<Arc<crate::domain_computation::vocabulary::primary_graph::inspection::OriginalSourceInspectionDisclosure>>{
        self.original_inspection_disclosure.clone()
    }

    pub(in crate::domain_computation::primary_graph) fn corrupt_original_disclosure(
        &mut self,
        disclosure: Arc<crate::domain_computation::vocabulary::primary_graph::inspection::OriginalSourceInspectionDisclosure>,
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
