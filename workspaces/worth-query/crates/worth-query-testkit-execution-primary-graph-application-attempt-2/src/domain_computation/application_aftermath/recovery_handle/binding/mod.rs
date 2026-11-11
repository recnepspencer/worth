use worth_relational::facade::history::BranchId;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationIdempotencyBinding;
use worth_query_installation::facade::WorthQueryInstalledAftermathContract;
use worth_query_execution::facade::domain_computation::WorthQueryOperationScopeBinding;
use worth_query_execution::facade::domain_computation::WorthQueryRecoveryHandleBinding;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::commit_authority_binding::WorthQueryRetainedGovernedInput;
pub struct WorthQueryRecoveryHandleBindingAxisProbe {
    pub binding: WorthQueryRecoveryHandleBinding,
}

impl WorthQueryRecoveryHandleBindingAxisProbe {
    pub fn real() -> Self {
        static BASELINE: std::sync::OnceLock<WorthQueryRecoveryHandleBinding> =
            std::sync::OnceLock::new();
        let binding = BASELINE
            .get_or_init(|| {
                let receipt =
                    crate::domain_computation::primary_graph::tests::recoverable_commit_support::committed_recoverable_application();
                WorthQueryRecoveryHandleBinding::from_receipt_for_test(&receipt, Some(u64::MAX))
                    .expect("the real fixture operation admits recovery")
            })
            .clone();
        Self { binding }
    }

    pub fn from_binding(binding: WorthQueryRecoveryHandleBinding) -> Self {
        Self { binding }
    }

    pub fn runtime_instance_id(mut self, value: u64) -> Self {
        (*self.binding.runtime_instance_id_mut_for_test()) = value;
        self
    }

    pub fn schema_identity(mut self, value: [u8; 32]) -> Self {
        (*self.binding.schema_identity_mut_for_test()) = value;
        self
    }

    pub fn branch(mut self, value: BranchId) -> Self {
        self.binding.commit_mut_for_test().branch_id = value;
        self
    }

    pub fn application_binding_generation(mut self, value: u64) -> Self {
        (*self.binding.application_binding_generation_mut_for_test()) = value;
        self
    }

    pub fn installed_operation(mut self, value: [u8; 32]) -> Self {
        (*self.binding.installed_operation_mut_for_test()) = value;
        self
    }

    pub fn retained_governed_input_identity(mut self, value: Option<[u8; 32]>) -> Self {
        (*self.binding.retained_governed_input_mut_for_test()) =
            value.map(WorthQueryRetainedGovernedInput::axis_probe_for_test);
        self
    }

    pub fn principal_scope(mut self, value: WorthQueryOperationScopeBinding) -> Self {
        (*self.binding.principal_scope_mut_for_test()) = value;
        self
    }

    pub fn idempotency(mut self, value: WorthQueryApplicationIdempotencyBinding) -> Self {
        (*self.binding.idempotency_mut_for_test()) = value;
        self
    }

    pub fn installed_aftermath(mut self, value: WorthQueryInstalledAftermathContract) -> Self {
        (*self.binding.installed_aftermath_mut_for_test()) = value;
        self
    }

    pub fn without_dispatch_outbox(mut self) -> Self {
        (*self.binding.committed_dispatch_outbox_mut_for_test()) = None;
        self
    }

    pub fn expires_at_unix_ms(mut self, value: Option<u64>) -> Self {
        (*self.binding.expires_at_unix_ms_mut_for_test()) = value;
        self
    }

    pub fn finish(self) -> WorthQueryRecoveryHandleBinding {
        self.binding
    }
}
