impl crate::BankIdentityRuntime {
    pub(crate) fn select_current_product(
        &self,
    ) -> Result<
        worth_query_host::facade::primary_graph::WorthQuerySelectedProductOperation<
            '_,
            bank_domain::schema::BankSchema,
        >,
        worth_query_host::facade::product::WorthQueryProductBranchAdmissionDenial,
    > {
        let application = self.application_runtime();
        application.on_branch(application.current_world()).select()
    }
}

pub(crate) fn bank_operation_scope_binding(
    binding: &worth_query_host::facade::primary_graph::WorthQueryOperationScopeBinding,
) -> bank_domain::proposals::BankOperationScopeBinding {
    let schema = binding.binding_identity();
    let principal = binding.principal();
    let scope = binding.scope();
    bank_domain::proposals::BankOperationScopeBinding::new(
        binding.runtime_authority(),
        bank_domain::proposals::BankOperationScopeSchemaBinding::new(
            schema.runtime_ordinal(),
            schema.generation(),
            *schema.package_identity().bytes(),
            *schema.schema_identity().bytes(),
        ),
        binding.operation_authority_identity(),
        bank_domain::proposals::BankOperationScopeEntityBinding::new(
            principal.partition_id(),
            principal.local_slot(),
            principal.generation(),
        ),
        bank_domain::proposals::BankOperationScopeEntityBinding::new(
            scope.partition_id(),
            scope.local_slot(),
            scope.generation(),
        ),
    )
}
