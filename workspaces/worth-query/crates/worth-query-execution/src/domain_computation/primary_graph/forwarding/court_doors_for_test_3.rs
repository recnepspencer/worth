//! Thin forwarding shims for `court_doors_for_test` methods onto `worth-query-execution-query`.

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_runtime_certification_controls_owned_publication_doors {
    use crate::basis::WorthQueryProductBranch;
    use crate::domain_computation::primary_graph::WorthQueryAdvancementDenial;
    use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
    use worth_query_installation::facade::ApplicationSchema;
    use worth_relational::facade::identity::EntityId;
    use worth_relational::facade::transactions::TransactionCommitError;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<S: ApplicationSchema + 'static> WorthQueryPrimaryGraphApplicationRuntime<S> {
        #[doc(hidden)]
        pub fn probe_owned_edge_retarget_for_test(&self, branch: WorthQueryProductBranch, owner: EntityId, target: EntityId, request: &WorthQueryRequestScope) -> Result<Result<(), TransactionCommitError>, WorthQueryAdvancementDenial> {
            self.inner.probe_owned_edge_retarget_for_test(branch, owner, target, request)
        }
        #[doc(hidden)]
        pub fn probe_current_ownership_after_merge_for_test(&self, branch: WorthQueryProductBranch, other: WorthQueryProductBranch, owner: EntityId, request: &WorthQueryRequestScope) -> Result<
            Result<
                (
                    worth_relational::facade::history::RelationalCommitReceipt,
                    Result<(), TransactionCommitError>,
                ),
                TransactionCommitError,
            >,
            WorthQueryAdvancementDenial,
        > {
            self.inner.probe_current_ownership_after_merge_for_test(branch, other, owner, request)
        }
        #[doc(hidden)]
        pub fn probe_current_metadata_writes_for_test(&self, branch: WorthQueryProductBranch, owner: EntityId, request: &WorthQueryRequestScope) -> Result<
            Vec<(
                worth_relational::facade::identity::KindId,
                u8,
                Result<(), TransactionCommitError>,
            )>,
            WorthQueryAdvancementDenial,
        > {
            self.inner.probe_current_metadata_writes_for_test(branch, owner, request)
        }
        #[doc(hidden)]
        pub fn current_owned_aggregate_memberships_for_test(&self, branch: WorthQueryProductBranch, owner: EntityId, request: &WorthQueryRequestScope) -> Result<Vec<worth_relational::facade::identity::RelationId>, WorthQueryAdvancementDenial> {
            self.inner.current_owned_aggregate_memberships_for_test(branch, owner, request)
        }
        #[doc(hidden)]
        pub fn probe_coherent_ownership_loss_for_test(&self, branch: WorthQueryProductBranch, owner: EntityId, request: &WorthQueryRequestScope) -> Result<Result<(), TransactionCommitError>, WorthQueryAdvancementDenial> {
            self.inner.probe_coherent_ownership_loss_for_test(branch, owner, request)
        }
        #[doc(hidden)]
        #[must_use = "retain the readonly recording guard until settlement completes"]
        pub fn record_output_publications_for_test(&self, observer: impl Fn(&crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt)
                + 'static) -> impl Sized {
            self.inner.record_output_publications_for_test(observer)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_runtime_certification_controls_primary_index_generations_doors {
    use worth_query_installation::facade::ApplicationSchema;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema: ApplicationSchema + 'static> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        #[doc(hidden)]
        pub fn release_primary_index_generations_for_test(&self, phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>) -> Result<usize, crate::domain_computation::primary_graph::WorthQueryAdvancementDenial> {
            self.inner.release_primary_index_generations_for_test(phase)
        }
        #[doc(hidden)]
        pub fn release_equality_index_generations_for_test<
            F: worth_query_declaration::facade::application_schema::fragment::RecordSelection,
        >(&self, phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>, field: F) -> Result<usize, crate::domain_computation::primary_graph::WorthQueryAdvancementDenial> {
            self.inner.release_equality_index_generations_for_test::<F>(phase, field)
        }
        #[doc(hidden)]
        pub fn has_equality_index_generation_for_test<
            F: worth_query_declaration::facade::application_schema::fragment::RecordSelection,
        >(&self, phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>, branch: crate::basis::WorthQueryProductBranch, field: F) -> Result<bool, crate::domain_computation::primary_graph::WorthQueryAdvancementDenial> {
            self.inner.has_equality_index_generation_for_test::<F>(phase, branch, field)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_runtime_certification_controls_provider_session_grant_doors {
    use crate::domain_computation::WorthQueryProviderSessionGrantControlDenial;
    use crate::domain_computation::WorthQueryProviderSessionGrantControlForTest;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        #[doc(hidden)]
        pub fn capture_next_provider_session_grant_for_test(&self) -> Result<
            WorthQueryProviderSessionGrantControlForTest,
            WorthQueryProviderSessionGrantControlDenial,
        > {
            self.inner.capture_next_provider_session_grant_for_test()
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_runtime_certification_controls_publication_doors {
    use crate::domain_computation::primary_graph::provider::publication_fault_control::WorthQueryPublicationFaultGuardForTest;
    use crate::domain_computation::primary_graph::provider::publication_fault_control::WorthQueryPublicationFaultRegistrationDenial;
    use worth_query_installation::facade::ApplicationSchema;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<S: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<S> {
        pub fn probe_next_discovery_support_retention_memory_for_test(&self) -> Result<WorthQueryPublicationFaultGuardForTest, WorthQueryPublicationFaultRegistrationDenial> {
            self.inner.probe_next_discovery_support_retention_memory_for_test()
        }
        pub fn reject_next_commit_before_transaction_for_test(&self) -> Result<WorthQueryPublicationFaultGuardForTest, WorthQueryPublicationFaultRegistrationDenial> {
            self.inner.reject_next_commit_before_transaction_for_test()
        }
        pub fn hold_post_commit_snapshot_failure_for_test(&self) -> Result<WorthQueryPublicationFaultGuardForTest, WorthQueryPublicationFaultRegistrationDenial> {
            self.inner.hold_post_commit_snapshot_failure_for_test()
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_runtime_certification_controls_signal_publication_doors {
    use crate::basis::WorthQueryProductBranch;
    use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
    use worth_query_installation::facade::ApplicationSchema;
    use worth_runtime_world::facade::ConsumedCompositePublication;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        #[doc(hidden)]
        pub fn prepare_signal_only_publication_for_test<'runtime>(&'runtime self, branch: WorthQueryProductBranch, request: &'runtime WorthQueryRequestScope) -> impl FnOnce() -> ConsumedCompositePublication + 'runtime {
            self.inner.prepare_signal_only_publication_for_test(branch, request)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_runtime_certification_controls_workflow_approval_doors {
    use worth_query_installation::facade::ApplicationSchema;
    use worth_relational::facade::identity::EntityId;
    use worth_relational::facade::transactions::TransactionCommitError;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema + 'static, {
        #[doc(hidden)]
        pub fn attempt_workflow_approval_field_update_for_test(&self, phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>, instance: &crate::domain_computation::primary_graph::PublishedWorkflowInstanceRef, approval: EntityId) -> Result<
            Result<(), TransactionCommitError>,
            crate::domain_computation::primary_graph::WorthQueryForeignAdvancementPhase,
        > {
            self.inner.attempt_workflow_approval_field_update_for_test(phase, instance, approval)
        }
        #[doc(hidden)]
        pub fn attempt_workflow_approval_evidence_delete_for_test(&self, phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>, instance: &crate::domain_computation::primary_graph::PublishedWorkflowInstanceRef, approval: EntityId) -> Result<
            Result<(), TransactionCommitError>,
            crate::domain_computation::primary_graph::WorthQueryForeignAdvancementPhase,
        > {
            self.inner.attempt_workflow_approval_evidence_delete_for_test(phase, instance, approval)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_runtime_certification_controls_workflow_definition_doors {
    use worth_foundational::facade::AspectValue;
    use worth_query_installation::facade::ApplicationSchema;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema + 'static, {
        #[doc(hidden)]
        pub fn workflow_definition_revisions_for_test(&self, phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>, definition: &crate::domain_computation::primary_graph::PublishedWorkflowDefinitionRef) -> Result<
            Result<
                (Option<AspectValue>, Option<AspectValue>),
                worth_relational::facade::transactions::RelationalExecutionDenialCause,
            >,
            crate::domain_computation::primary_graph::WorthQueryAdvancementDenial,
        > {
            self.inner.workflow_definition_revisions_for_test(phase, definition)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_runtime_certification_controls_workflow_membership_doors {
    use worth_query_installation::facade::ApplicationSchema;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema + 'static, {
        #[doc(hidden)]
        pub fn attempt_workflow_definition_membership_cycle_for_test(&self, phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>, definition: &crate::domain_computation::primary_graph::PublishedWorkflowDefinitionRef) -> Result<
            Result<(), worth_relational::facade::transactions::TransactionCommitError>,
            crate::domain_computation::primary_graph::WorthQueryForeignAdvancementPhase,
        > {
            self.inner.attempt_workflow_definition_membership_cycle_for_test(phase, definition)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_runtime_certification_controls_owned_publication_inspection_doors {
    use crate::basis::WorthQueryProductBranch;
    use crate::domain_computation::primary_graph::WorthQueryAdvancementDenial;
    use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
    use worth_query_installation::facade::ApplicationSchema;
    use worth_relational::facade::identity::EntityId;
    use worth_relational::facade::transactions::TransactionCommitError;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<S: ApplicationSchema + 'static> WorthQueryPrimaryGraphApplicationRuntime<S> {
        #[doc(hidden)]
        pub fn current_ownership_sources_for_record_for_test(&self, record: EntityId, request: &WorthQueryRequestScope) -> Result<Vec<EntityId>, WorthQueryAdvancementDenial> {
            self.inner.current_ownership_sources_for_record_for_test(record, request)
        }
        #[doc(hidden)]
        pub fn current_ownership_contacts_for_record_for_test(&self, record: EntityId, request: &WorthQueryRequestScope) -> Result<
            worth_relational::facade::runtime::CurrentOwnershipContactsForTest,
            WorthQueryAdvancementDenial,
        > {
            self.inner.current_ownership_contacts_for_record_for_test(record, request)
        }
        #[doc(hidden)]
        pub fn probe_unmatched_ownership_presence_for_test(&self, branch: WorthQueryProductBranch, owner: EntityId, request: &WorthQueryRequestScope) -> Result<Result<(), TransactionCommitError>, WorthQueryAdvancementDenial> {
            self.inner.probe_unmatched_ownership_presence_for_test(branch, owner, request)
        }
        #[doc(hidden)]
        pub fn current_ownership_contacts_for_identity_for_test(&self, identity: worth_runtime_bridge::facade::RelationalBridgeRecordIdentityParts, request: &WorthQueryRequestScope) -> Result<
            worth_relational::facade::runtime::CurrentOwnershipContactsForTest,
            WorthQueryAdvancementDenial,
        > {
            self.inner.current_ownership_contacts_for_identity_for_test(identity, request)
        }
        #[doc(hidden)]
        pub fn probe_current_history_contradiction_for_test(&self, publication: worth_relational::facade::history::CommitId, source: EntityId, request: &WorthQueryRequestScope) -> Result<
            Result<Vec<Result<(), TransactionCommitError>>, TransactionCommitError>,
            WorthQueryAdvancementDenial,
        > {
            self.inner.probe_current_history_contradiction_for_test(publication, source, request)
        }
        #[doc(hidden)]
        pub fn probe_recorded_adjacency_support_for_test(&self, original: &worth_relational::facade::history::BranchId, anchor: EntityId, kind: worth_relational::facade::identity::KindId, request: &WorthQueryRequestScope) -> Result<
            Result<(), worth_relational::facade::history::RelationalAuthoredWriteDenial>,
            WorthQueryAdvancementDenial,
        > {
            self.inner.probe_recorded_adjacency_support_for_test(original, anchor, kind, request)
        }
        #[doc(hidden)]
        pub fn recorded_support_source_branch_for_test(&self, branch: WorthQueryProductBranch, request: &WorthQueryRequestScope) -> Result<worth_relational::facade::history::BranchId, WorthQueryAdvancementDenial> {
            self.inner.recorded_support_source_branch_for_test(branch, request)
        }
        #[doc(hidden)]
        pub fn current_ownership_descendants_for_record_for_test(&self, source: EntityId, request: &WorthQueryRequestScope) -> Result<Vec<EntityId>, WorthQueryAdvancementDenial> {
            self.inner.current_ownership_descendants_for_record_for_test(source, request)
        }
        #[doc(hidden)]
        pub fn record_current_ownership_contacts_for_test(&self, request: &WorthQueryRequestScope, observer: impl Fn(worth_relational::facade::runtime::CurrentOwnershipContactsForTest) + 'static) -> Result<impl Sized, WorthQueryAdvancementDenial> {
            self.inner.record_current_ownership_contacts_for_test(request, observer)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_runtime_inbound_occurrence_transport_publication_permit_court_doors_for_test_doors {
    use worth_query_installation::facade::ApplicationSchema;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_runtime::inbound_occurrence::WorthQueryInstalledTransportCompletionBinding;
    use crate::domain_computation::application_aftermath::WorthQueryTransportPublicationPermit;
    use crate::domain_computation::application_aftermath::WorthQueryTransportPublicationPermitDenial;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        #[doc(hidden)]
        pub fn reserve_installed_transport_publication_for_test(&self, binding: &WorthQueryInstalledTransportCompletionBinding) -> Result<WorthQueryTransportPublicationPermit, WorthQueryTransportPublicationPermitDenial> {
            self.inner.reserve_installed_transport_publication_for_test(binding)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_runtime_inbound_occurrence_progress_court_doors_for_test_doors {
    use std::sync::Arc;
    use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
    use worth_query_installation::facade::ApplicationSchema;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_runtime::inbound_occurrence::receive::WorthQueryInboundAdmissionDenial;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_runtime::inbound_occurrence::receive::WorthQueryInboundReceiptPosture;
    use crate::domain_computation::application_aftermath::WorthQueryAcceptedInboundOccurrence;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        #[doc(hidden)]
        pub fn progress_accepted_inbound_occurrence_for_test(&self, execution: &worth_execution::ExecutionRequest<'_, '_>, phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>, accepted: Arc<WorthQueryAcceptedInboundOccurrence>, request: &WorthQueryRequestScope) -> Result<WorthQueryInboundReceiptPosture, WorthQueryInboundAdmissionDenial> {
            self.inner.progress_accepted_inbound_occurrence_for_test(execution, phase, accepted, request)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_runtime_inbound_occurrence_reconstruct_court_doors_for_test_doors {
    use crate::domain_computation::application_aftermath::ExternalEffectCorrelationIdentity;
    use crate::domain_computation::primary_graph::provider::WorthQueryInboundTerminalIndexDenial as Denial;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        #[doc(hidden)]
        pub fn verify_completed_inbound_for_cleanup_for_test(&self, correlations: &[ExternalEffectCorrelationIdentity], now_unix_seconds: u64) -> Result<(), Denial> {
            self.inner.verify_completed_inbound_for_cleanup_for_test(correlations, now_unix_seconds)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_primary_graph_application_runtime_inbound_occurrence_receive_admit_court_doors_for_test_doors {
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_runtime::inbound_occurrence::WorthQueryInboundVerifierHandle;
    use worth_query_execution_attempt::facade::domain_computation::primary_graph::application_runtime::inbound_occurrence::WorthQueryInboundAdmissionDenial;
    use worth_query_execution_query::facade::domain_computation::primary_graph::application_runtime::inbound_occurrence::receive::WorthQueryInboundAdmission;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: worth_query_installation::facade::ApplicationSchema, {
        #[doc(hidden)]
        pub fn admit_inbound_occurrence_for_test(&self, handle: &WorthQueryInboundVerifierHandle, envelope: &[u8]) -> Result<WorthQueryInboundAdmission, WorthQueryInboundAdmissionDenial> {
            self.inner.admit_inbound_occurrence_for_test(handle, envelope)
        }
    }
}
