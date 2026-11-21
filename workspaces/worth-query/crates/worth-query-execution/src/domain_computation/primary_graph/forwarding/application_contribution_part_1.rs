//! Thin forwarding shims for `application_contribution` methods onto `worth-query-execution-query`.

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_contribution_producer_demand_durable {
    use crate::facade::product::WorthQueryProductBranch;
    use worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal;
    use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
    use worth_query_installation::facade::ApplicationSchema;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_contribution::producer::WorthQueryProducerOutputFamily;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_contribution::producer::demand::durable::WorthQueryOriginalOutputDemand;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_contribution::producer::demand::denial::WorthQueryOutputDemandDenial;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_contribution::producer::demand::WorthQueryAdmittedOutputDemand;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    type FamilySourceValue<Schema, Family> =
    <<FamilySource<Schema, Family> as worth_query_declaration::facade::application_query::ApplicationQueryBinding<Schema>>::ResultBinding as worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding>::Value;
    type FamilySourceQuery<Schema, Family> =
    <FamilySource<Schema, Family> as worth_query_declaration::facade::application_query::ApplicationQueryBinding<Schema>>::Query;
    type FamilySource<Schema, Family> = <Family as WorthQueryProducerOutputFamily<Schema>>::Source;
    impl<S: ApplicationSchema + 'static> WorthQueryPrimaryGraphApplicationRuntime<S> {
        pub fn readmit_original_output_demand<F>(&self, phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>, principal: &WorthQueryAuthenticatedExternalPrincipal<S>, request: &WorthQueryRequestScope, branch: WorthQueryProductBranch, original: &WorthQueryOriginalOutputDemand) -> Result<WorthQueryAdmittedOutputDemand<S, F>, WorthQueryOutputDemandDenial>
        where
            F: WorthQueryProducerOutputFamily<S>,
            FamilySourceValue<S, F>:
                crate::domain_computation::primary_graph::WorthQueryApplicationProjection<
                        S,
                        FamilySourceQuery<S, F>,
                    > + 'static,
            FamilySourceQuery<S, F>: 'static, {
            self.inner.readmit_original_output_demand::<F>(phase, principal, request, branch, original)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_contribution_producer_demand_progression {
    use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
    use worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal;
    use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
    use worth_query_installation::facade::ApplicationSchema;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_contribution::producer::demand::FamilySourceQuery;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_contribution::producer::demand::FamilySourceValue;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_contribution::producer::demand::WorthQueryAdmittedOutputDemand;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_contribution::producer::demand::WorthQueryOutputDemandAdvance;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_contribution::producer::demand::WorthQueryOutputDemandDenial;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_contribution::producer::demand::WorthQueryProducerOutputFamily;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema + 'static, {
        pub fn advance_output_demand<Family>(&self, phase: &WorthQueryAdvancementPhase<'_>, demand: &mut WorthQueryAdmittedOutputDemand<Schema, Family>, principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>, request_scope: &WorthQueryRequestScope, delivery_branch: crate::basis::WorthQueryProductBranch, disclosure: crate::domain_computation::primary_graph::WorthQueryApplicationOutputDemandSource<
                FamilySourceQuery<Schema, Family>,
                FamilySourceValue<Schema, Family>,
            >) -> Result<WorthQueryOutputDemandAdvance, WorthQueryOutputDemandDenial>
        where
            Family: WorthQueryProducerOutputFamily<Schema>,
            FamilySourceValue<Schema, Family>: 'static,
            FamilySourceQuery<Schema, Family>: 'static, {
            self.inner.advance_output_demand::<Family>(phase, demand, principal, request_scope, delivery_branch, disclosure)
        }
        pub fn advance_output_demand_from_retained<Family>(&self, phase: &WorthQueryAdvancementPhase<'_>, demand: &mut WorthQueryAdmittedOutputDemand<Schema, Family>, principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>, request_scope: &WorthQueryRequestScope, delivery_branch: crate::basis::WorthQueryProductBranch) -> Result<WorthQueryOutputDemandAdvance, WorthQueryOutputDemandDenial>
        where
            Family: WorthQueryProducerOutputFamily<Schema>,
            FamilySourceValue<Schema, Family>:
                crate::domain_computation::primary_graph::WorthQueryApplicationProjection<
                        Schema,
                        FamilySourceQuery<Schema, Family>,
                    > + 'static,
            FamilySourceQuery<Schema, Family>: 'static, {
            self.inner.advance_output_demand_from_retained::<Family>(phase, demand, principal, request_scope, delivery_branch)
        }
        #[allow(clippy::too_many_arguments)]
        pub(crate) fn advance_program_output_demand<Family>(&self, execution: &worth_execution::ExecutionRequest<'_, '_>, phase: &WorthQueryAdvancementPhase<'_>, demand: &mut WorthQueryAdmittedOutputDemand<Schema, Family>, principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>, request_scope: &WorthQueryRequestScope, delivery_branch: crate::basis::WorthQueryProductBranch, disclosure: crate::domain_computation::primary_graph::WorthQueryApplicationOutputDemandSource<
                FamilySourceQuery<Schema, Family>,
                FamilySourceValue<Schema, Family>,
            >) -> Result<WorthQueryOutputDemandAdvance, WorthQueryOutputDemandDenial>
        where
            Family: WorthQueryProducerOutputFamily<Schema>,
            FamilySourceValue<Schema, Family>: 'static,
            FamilySourceQuery<Schema, Family>: 'static, {
            self.inner.advance_program_output_demand::<Family>(execution, phase, demand, principal, request_scope, delivery_branch, disclosure)
        }
        #[allow(clippy::too_many_arguments)]
        pub(crate) fn advance_history_output_demand<Family>(&self, execution: &worth_execution::ExecutionRequest<'_, '_>, phase: &WorthQueryAdvancementPhase<'_>, demand: &mut WorthQueryAdmittedOutputDemand<Schema, Family>, principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>, request_scope: &WorthQueryRequestScope, delivery_branch: crate::basis::WorthQueryProductBranch, history: crate::domain_computation::primary_graph::product_operation::document_history::HistoryProducerInvocation<'_>) -> Result<WorthQueryOutputDemandAdvance, WorthQueryOutputDemandDenial>
        where
            Family: WorthQueryProducerOutputFamily<Schema>,
            FamilySourceValue<Schema, Family>:
                crate::domain_computation::primary_graph::WorthQueryApplicationProjection<
                        Schema,
                        FamilySourceQuery<Schema, Family>,
                    > + 'static,
            FamilySourceQuery<Schema, Family>: 'static, {
            self.inner.advance_history_output_demand::<Family>(execution, phase, demand, principal, request_scope, delivery_branch, history)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_contribution_producer_demand_selection {
    use worth_query_installation::facade::ApplicationSchema;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_contribution::producer::WorthQueryProducerOutputFamily;
    use crate::domain_computation::primary_graph::WorthQueryObservedSource;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_contribution::producer::demand::denial::WorthQueryOutputDemandDenial;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_contribution::producer::demand::WorthQuerySelectedApplicationProducer;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema + 'static, {
        pub fn select_output_producer<Family>(&self, phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>, source: &WorthQueryObservedSource<
                <<Family as WorthQueryProducerOutputFamily<Schema>>::Source as worth_query_declaration::facade::application_query::ApplicationQueryBinding<Schema>>::Query,
            >, profile_kind: &'static str) -> Result<WorthQuerySelectedApplicationProducer, WorthQueryOutputDemandDenial>
        where
            Family: WorthQueryProducerOutputFamily<Schema>, {
            self.inner.select_output_producer::<Family>(phase, source, profile_kind)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_contribution_producer_demand_progression_admission {
    use worth_query_installation::facade::ApplicationSchema;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_contribution::producer::demand::FamilySourceQuery;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_contribution::producer::demand::FamilySourceValue;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_contribution::producer::demand::WorthQueryAdmittedOutputDemand;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_contribution::producer::demand::WorthQueryOutputDemandDenial;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_contribution::producer::demand::WorthQueryProducerOutputFamily;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema + 'static, {
        pub fn output_demand_resource_profile(&self) -> crate::domain_computation::execution_runtime::WorthQueryOutputDemandResourceProfile {
            self.inner.output_demand_resource_profile()
        }
        pub fn admit_output_demand<Family>(&self, phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>, source_result: crate::domain_computation::primary_graph::WorthQueryApplicationOutputDemandSource<
                FamilySourceQuery<Schema, Family>,
                FamilySourceValue<Schema, Family>,
            >, limits: crate::domain_computation::execution_runtime::WorthQueryOutputDemandLimits) -> Result<WorthQueryAdmittedOutputDemand<Schema, Family>, WorthQueryOutputDemandDenial>
        where
            Family: WorthQueryProducerOutputFamily<Schema>, {
            self.inner.admit_output_demand::<Family>(phase, source_result, limits)
        }
        #[doc(hidden)]
        pub(crate) fn admit_required_output_demand<Family>(&self, execution: &worth_execution::ExecutionRequest<'_, '_>, source_result: crate::domain_computation::primary_graph::WorthQueryApplicationOutputDemandSource<
                FamilySourceQuery<Schema, Family>,
                FamilySourceValue<Schema, Family>,
            >, limits: crate::domain_computation::execution_runtime::WorthQueryOutputDemandLimits) -> Result<WorthQueryAdmittedOutputDemand<Schema, Family>, WorthQueryOutputDemandDenial>
        where
            Family: WorthQueryProducerOutputFamily<Schema>, {
            self.inner.admit_required_output_demand::<Family>(execution, source_result, limits)
        }
        #[doc(hidden)]
        pub(crate) fn admit_recovered_output_demand<Family>(&self, execution: &worth_execution::ExecutionRequest<'_, '_>, source_result: crate::domain_computation::primary_graph::WorthQueryApplicationOutputDemandSource<
                FamilySourceQuery<Schema, Family>,
                FamilySourceValue<Schema, Family>,
            >, current_result: crate::domain_computation::primary_graph::WorthQueryApplicationOutputDemandSource<
                FamilySourceQuery<Schema, Family>,
                FamilySourceValue<Schema, Family>,
            >, limits: crate::domain_computation::execution_runtime::WorthQueryOutputDemandLimits, source_receipt: &crate::domain_computation::primary_graph::WorthQueryOutputSourceEvidence) -> Result<WorthQueryAdmittedOutputDemand<Schema, Family>, WorthQueryOutputDemandDenial>
        where
            Family: WorthQueryProducerOutputFamily<Schema>, {
            self.inner.admit_recovered_output_demand::<Family>(execution, source_result, current_result, limits, source_receipt)
        }
        pub fn admit_performed_output_demand<Family>(&self, execution: &worth_execution::ExecutionRequest<'_, '_>, source_result: crate::domain_computation::primary_graph::WorthQueryApplicationOutputDemandSource<
                FamilySourceQuery<Schema, Family>,
                FamilySourceValue<Schema, Family>,
            >, limits: crate::domain_computation::execution_runtime::WorthQueryOutputDemandLimits, prepared: &crate::domain_computation::primary_graph::WorthQueryPreparedRequiredOutputSource, retained_program_basis: std::sync::Arc<
                crate::domain_computation::primary_graph::WorthQueryApplicationReadObservation,
            >) -> Result<WorthQueryAdmittedOutputDemand<Schema, Family>, WorthQueryOutputDemandDenial>
        where
            Family: WorthQueryProducerOutputFamily<Schema>, {
            self.inner.admit_performed_output_demand::<Family>(execution, source_result, limits, prepared, retained_program_basis)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_contribution_producer_demand_progression_durable_output {
    use worth_query_installation::facade::ApplicationSchema;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_contribution::producer::demand::WorthQueryOutputDemandDenial;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<S: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<S> {
        pub(crate) fn retain_coupled_output_records(&self, execution: &worth_execution::ExecutionRequest<'_, '_>, receipt: &crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt, observation: &worth_runtime_world::facade::ProductBranchObservation) -> Result<(), WorthQueryOutputDemandDenial> {
            self.inner.retain_coupled_output_records(execution, receipt, observation)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_contribution_producer_demand_progression_selected_program {
    use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
    use worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal;
    use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
    use worth_query_declaration::facade::application_program::ApplicationProgramIdentity;
    use worth_query_declaration::facade::application_program::ApplicationProgramRevision;
    use worth_query_installation::facade::ApplicationSchema;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_contribution::producer::demand::progression::FamilySourceQuery;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_contribution::producer::demand::progression::FamilySourceValue;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_contribution::producer::demand::progression::WorthQueryAdmittedOutputDemand;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_contribution::producer::demand::progression::WorthQueryOutputDemandAdvance;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_contribution::producer::demand::progression::WorthQueryOutputDemandDenial;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_contribution::producer::demand::progression::WorthQueryProducerOutputFamily;
    use crate::domain_computation::primary_graph::WorthQueryApplicationOutputDemandSource;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema + 'static, {
        #[allow(clippy::too_many_arguments)]
        pub fn advance_selected_program_output_demand<Family>(&self, execution: &worth_execution::ExecutionRequest<'_, '_>, phase: &WorthQueryAdvancementPhase<'_>, access: &crate::publication_boundary::WorthQueryProgramPublicationAccess, demand: &mut WorthQueryAdmittedOutputDemand<Schema, Family>, principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>, request_scope: &WorthQueryRequestScope, delivery_branch: crate::basis::WorthQueryProductBranch, disclosure: WorthQueryApplicationOutputDemandSource<
                FamilySourceQuery<Schema, Family>,
                FamilySourceValue<Schema, Family>,
            >, identity: ApplicationProgramIdentity, revision: ApplicationProgramRevision) -> Result<WorthQueryOutputDemandAdvance, WorthQueryOutputDemandDenial>
        where
            Family: WorthQueryProducerOutputFamily<Schema>,
            FamilySourceValue<Schema, Family>: 'static,
            FamilySourceQuery<Schema, Family>: 'static, {
            self.inner.advance_selected_program_output_demand::<Family>(execution, phase, access, demand, principal, request_scope, delivery_branch, disclosure, identity, revision)
        }
        #[allow(clippy::too_many_arguments)]
        pub fn advance_selected_program_output_demand_from_retained<Family>(&self, phase: &WorthQueryAdvancementPhase<'_>, access: &crate::publication_boundary::WorthQueryProgramPublicationAccess, demand: &mut WorthQueryAdmittedOutputDemand<Schema, Family>, principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>, request_scope: &WorthQueryRequestScope, delivery_branch: crate::basis::WorthQueryProductBranch, identity: ApplicationProgramIdentity, revision: ApplicationProgramRevision) -> Result<WorthQueryOutputDemandAdvance, WorthQueryOutputDemandDenial>
        where
            Family: WorthQueryProducerOutputFamily<Schema>,
            FamilySourceValue<Schema, Family>:
                crate::domain_computation::primary_graph::WorthQueryApplicationProjection<
                        Schema,
                        FamilySourceQuery<Schema, Family>,
                    > + 'static,
            FamilySourceQuery<Schema, Family>: 'static, {
            self.inner.advance_selected_program_output_demand_from_retained::<Family>(phase, access, demand, principal, request_scope, delivery_branch, identity, revision)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_contribution_producer_demand_progression_source_recovery {
    use crate::equality_type_id::EqualityTypeId as TypeId;
    use worth_query_installation::facade::ApplicationSchema;
    use crate::domain_computation::primary_graph::WorthQueryApplicationOutputDemandSource;
    use crate::domain_computation::primary_graph::WorthQueryApplicationReadObservation;
    use crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial;
    use crate::domain_computation::primary_graph::WorthQueryOutputSourceEvidence;
    use crate::domain_computation::primary_graph::WorthQueryPreparedRequiredOutputSource;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema: ApplicationSchema + 'static> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        pub(crate) fn resolve_output_source_replay(&self, phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>, selector: crate::domain_computation::primary_graph::WorthQueryOutputSourceReplay<'_>, kind: crate::domain_computation::primary_graph::application_output_demand::PreparedOutputRootKind, root_identity: &str) -> Result<WorthQueryOutputSourceEvidence, WorthQueryOutputDemandDenial> {
            self.inner.resolve_output_source_replay(phase, selector, kind, root_identity)
        }
        pub(crate) fn validate_recovered_output_root_kind(&self, receipt: &WorthQueryOutputSourceEvidence, kind: crate::domain_computation::primary_graph::application_output_demand::PreparedOutputRootKind) -> Result<(), WorthQueryOutputDemandDenial> {
            self.inner.validate_recovered_output_root_kind(receipt, kind)
        }
        pub(crate) fn complete_prepared_output_source(&self, receipt: &WorthQueryOutputSourceEvidence) {
            self.inner.complete_prepared_output_source(receipt)
        }
        pub(crate) fn recover_prepared_output_source(&self, receipt: &WorthQueryOutputSourceEvidence, root_type: TypeId) -> Option<(
            std::sync::Arc<WorthQueryApplicationReadObservation>,
            WorthQueryPreparedRequiredOutputSource,
        )> {
            self.inner.recover_prepared_output_source(receipt, root_type)
        }
        pub(crate) fn ensure_recovered_output_source_bound<
            Query,
            Value,
        >(&self, phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>, prepared: &WorthQueryPreparedRequiredOutputSource, source: &WorthQueryApplicationOutputDemandSource<Query, Value>) -> Result<(), WorthQueryOutputDemandDenial> {
            self.inner.ensure_recovered_output_source_bound::<Query, Value>(phase, prepared, source)
        }
        pub(crate) fn validate_recovered_output_source_currentness<
            Query,
            Value,
        >(&self, prepared: &WorthQueryPreparedRequiredOutputSource, retained: &WorthQueryApplicationOutputDemandSource<Query, Value>, current: &WorthQueryApplicationOutputDemandSource<Query, Value>) -> Result<(), WorthQueryOutputDemandDenial> {
            self.inner.validate_recovered_output_source_currentness::<Query, Value>(prepared, retained, current)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_contribution_producer_demand_progression_unavailable {
    use crate::domain_computation::primary_graph::WorthQueryApplicationOutputDemandSource;
    use crate::domain_computation::primary_graph::WorthQueryPreparedRequiredOutputSource;
    use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
    use worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal;
    use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
    use worth_query_installation::facade::ApplicationSchema;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_contribution::producer::demand::FamilySourceQuery;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_contribution::producer::demand::FamilySourceValue;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_contribution::producer::demand::WorthQueryAdmittedOutputDemand;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_contribution::producer::demand::WorthQueryOutputDemandDenial;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_contribution::producer::demand::WorthQueryProducerOutputFamily;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema: ApplicationSchema + 'static> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        #[allow(clippy::too_many_arguments)]
        pub(crate) fn finish_unavailable_output_demand<Family>(&self, execution: &worth_execution::ExecutionRequest<'_, '_>, phase: &WorthQueryAdvancementPhase<'_>, demand: &mut WorthQueryAdmittedOutputDemand<Schema, Family>, prepared: &WorthQueryPreparedRequiredOutputSource, principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>, scope: &WorthQueryRequestScope, branch: crate::basis::WorthQueryProductBranch, disclosure: WorthQueryApplicationOutputDemandSource<
                FamilySourceQuery<Schema, Family>,
                FamilySourceValue<Schema, Family>,
            >) -> Result<WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenial>
        where
            Family: WorthQueryProducerOutputFamily<Schema>, {
            self.inner.finish_unavailable_output_demand::<Family>(execution, phase, demand, prepared, principal, scope, branch, disclosure)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_contribution_producer_demand_progression_admission_dependent {
    use worth_query_installation::facade::ApplicationSchema;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_contribution::producer::demand::FamilySourceQuery;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_contribution::producer::demand::FamilySourceValue;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_contribution::producer::demand::WorthQueryAdmittedOutputDemand;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_contribution::producer::demand::WorthQueryOutputDemandDenial;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_contribution::producer::demand::WorthQueryProducerOutputFamily;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema + 'static, {
        pub(crate) fn admit_dependent_output_demand<Family>(&self, execution: &worth_execution::ExecutionRequest<'_, '_>, source_result: crate::domain_computation::primary_graph::WorthQueryApplicationOutputDemandSource<
                FamilySourceQuery<Schema, Family>,
                FamilySourceValue<Schema, Family>,
            >, limits: crate::domain_computation::execution_runtime::WorthQueryOutputDemandLimits, retained_basis: std::sync::Arc<
                crate::domain_computation::primary_graph::WorthQueryApplicationReadObservation,
            >) -> Result<WorthQueryAdmittedOutputDemand<Schema, Family>, WorthQueryOutputDemandDenial>
        where
            Family: WorthQueryProducerOutputFamily<Schema>, {
            self.inner.admit_dependent_output_demand::<Family>(execution, source_result, limits, retained_basis)
        }
    }
}
