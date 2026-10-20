use super::*;

impl WorthQueryExecutionInstallationAuthority {
    pub fn prepare_primary_graph<Schema>(
        &self,
        phase: &super::super::WorthQueryBootstrapAdvancementPhase<'_>,
        runtime: &WorthQueryExecutionRuntime,
        installed_schema: &WorthQueryInstalledApplicationSchema<Schema>,
        product_world_resources: crate::domain_computation::execution_runtime::product_world::WorthQueryProductWorldResources,
    ) -> Result<WorthQueryPrimaryGraphBootstrap<Schema>, WorthQueryPrimaryGraphInstallationDenial>
    where
        Schema: ApplicationSchema,
    {
        // Installation has no World identity yet; this borrows its admitted root.
        let factories =
            WorthQueryApplicationInvariantFactories::for_installed_schema(installed_schema);
        self.prepare_primary_graph_with_relational_runtime_and_invariants(
            phase,
            runtime,
            installed_schema,
            RelationalRuntimeApi::builder().build(),
            product_world_resources,
            factories,
        )
    }

    pub(crate) fn prepare_primary_graph_with_relational_runtime<Schema>(
        &self,
        phase: &super::super::WorthQueryBootstrapAdvancementPhase<'_>,
        runtime: &WorthQueryExecutionRuntime,
        installed_schema: &WorthQueryInstalledApplicationSchema<Schema>,
        relational_runtime: RelationalRuntime,
        product_world_resources: crate::domain_computation::execution_runtime::product_world::WorthQueryProductWorldResources,
    ) -> Result<WorthQueryPrimaryGraphBootstrap<Schema>, WorthQueryPrimaryGraphInstallationDenial>
    where
        Schema: ApplicationSchema,
    {
        let factories =
            WorthQueryApplicationInvariantFactories::for_installed_schema(installed_schema);
        self.prepare_primary_graph_with_relational_runtime_and_invariants(
            phase,
            runtime,
            installed_schema,
            relational_runtime,
            product_world_resources,
            factories,
        )
    }

    pub(crate) fn prepare_primary_graph_with_relational_runtime_and_invariants<Schema>(
        &self,
        phase: &super::super::WorthQueryBootstrapAdvancementPhase<'_>,
        runtime: &WorthQueryExecutionRuntime,
        installed_schema: &WorthQueryInstalledApplicationSchema<Schema>,
        relational_runtime: RelationalRuntime,
        product_world_resources: crate::domain_computation::execution_runtime::product_world::WorthQueryProductWorldResources,
        invariant_factories: WorthQueryApplicationInvariantFactories<Schema>,
    ) -> Result<WorthQueryPrimaryGraphBootstrap<Schema>, WorthQueryPrimaryGraphInstallationDenial>
    where
        Schema: ApplicationSchema,
    {
        self.prepare_primary_graph_with_optional_checkpoint(
            phase,
            runtime,
            installed_schema,
            relational_runtime,
            product_world_resources,
            invariant_factories,
            None,
            false,
        )
    }

    /// The phase witnesses admission before inline Relational schema and recovery reads.
    pub(super) fn prepare_primary_graph_with_optional_checkpoint<Schema>(
        &self,
        phase: &super::super::WorthQueryBootstrapAdvancementPhase<'_>,
        runtime: &WorthQueryExecutionRuntime,
        installed_schema: &WorthQueryInstalledApplicationSchema<Schema>,
        mut relational_runtime: RelationalRuntime,
        product_world_resources: crate::domain_computation::execution_runtime::product_world::WorthQueryProductWorldResources,
        invariant_factories: WorthQueryApplicationInvariantFactories<Schema>,
        checkpoint: Option<&super::super::application_checkpoint::DecodedApplicationCheckpoint>,
        codec_transition: bool,
    ) -> Result<WorthQueryPrimaryGraphBootstrap<Schema>, WorthQueryPrimaryGraphInstallationDenial>
    where
        Schema: ApplicationSchema,
    {
        let execution = phase.execution_request();

        if !self.belongs_to(runtime) {
            return Err(primary_graph_denial(
                WorthQueryPrimaryGraphInstallationDenialKind::ForeignRuntime,
                "execution installation authority belongs to another runtime",
            ));
        }
        if runtime.primary_graph().is_some() {
            return Err(primary_graph_denial(
                WorthQueryPrimaryGraphInstallationDenialKind::AlreadyInstalled,
                "execution runtime already owns a primary graph",
            ));
        }
        if let Some((retained_positions, retained_commits)) =
            product_world_resources.invalidation_window_past_history()
        {
            return Err(primary_graph_denial(
                WorthQueryPrimaryGraphInstallationDenialKind::InvalidationWindowExceedsHistory {
                    retained_positions,
                    retained_commits,
                },
                "the invalidation window retains more positions than the World retains commits",
            ));
        }
        runtime
            .installed_packages()
            .validate_application_schema(installed_schema)
            .map_err(|denial| {
                primary_graph_denial(
                    WorthQueryPrimaryGraphInstallationDenialKind::StaleInstalledSchema,
                    denial.subject(),
                )
            })?;
        let (layout, additions) = WorthQueryPrimaryGraphLayout::lower(
            installed_schema.installed_declaration(),
            installed_schema.native_contracts(),
            &relational_runtime.config().schema.registry,
        )?;
        let registrations =
            invariant_factories.lower(installed_schema.binding_identity(), &layout)?;
        let expected_invariant_inventory_digest =
            worth_relational::facade::runtime::custom_invariant_inventory_digest(&registrations);
        let mut invariant_installation_receipt = relational_runtime
            .prepare_initial_schema_installation()
            .map_err(map_initial_schema_installation_denial)?
            .install_with_custom_invariants(additions, registrations)
            .map_err(map_initial_schema_installation_denial)?;
        if invariant_installation_receipt.custom_invariant_inventory_digest()
            != &expected_invariant_inventory_digest
        {
            return Err(primary_graph_denial(
                WorthQueryPrimaryGraphInstallationDenialKind::InvariantInstallationReceiptMismatch,
                "Relational installed an invariant inventory outside the application catalog",
            ));
        }
        let (recovered_relational_authority, recovered_checkpoint_restore_work) = match checkpoint {
            Some(checkpoint) => {
                let mut recovery = relational_runtime.durability_recovery();
                let (outcome, authority) = if codec_transition {
                    recovery.restore_native_checkpoint_for_codec_transition_with_authority(
                        &execution,
                        &checkpoint.native,
                    )
                } else {
                    recovery
                        .restore_native_checkpoint_with_authority(&execution, &checkpoint.native)
                }
                .map_err(map_checkpoint_recovery_stop)?;
                if !authority.codec_transition_pending() {
                    invariant_installation_receipt = relational_runtime
                    .readmit_recovered_initial_schema_installation(invariant_installation_receipt)
                    .map_err(|error| {
                        primary_graph_denial(
                            if error.kind() == worth_relational::facade::runtime::RelationalInitialSchemaInstallationDenialKind::RecoveredAuthorityMismatch {
                                WorthQueryPrimaryGraphInstallationDenialKind::UnsupportedRecoveredNativeAuthorityLayout
                            } else { WorthQueryPrimaryGraphInstallationDenialKind::CheckpointRecoveryRejected },
                            format!("recovered schema authority denied: {error}"),
                        )
                    })?;
                }
                (Some(authority), outcome.checkpoint_restore_work)
            }
            None => (None, None),
        };
        let codec_pending = recovered_relational_authority
            .as_ref()
            .is_some_and(|authority| authority.codec_transition_pending());
        let source_publisher = if codec_pending {
            let (owner, native) = relational_runtime
                .durability_recovery()
                .issue_recovered_codec_source_publication_owner(
                    &execution,
                    recovered_relational_authority
                        .as_ref()
                        .expect("pending recovery owns its source"),
                )
                .map_err(super::checkpoint_transition::codec::map_admission)?;
            Some(
                super::super::root::application_publication::ApplicationPublication::recovered(
                    owner, native,
                ),
            )
        } else {
            None
        };
        let mut graph = checkpoint::primary_graph_for_installation(
            runtime.authority_identity(),
            installed_schema.binding_identity(),
            layout,
            relational_runtime,
            product_world_resources.invalidation_resources(),
            checkpoint.is_some(),
        )?;
        if let Some(publisher) = source_publisher {
            graph.application_publication = Some(std::sync::Arc::new(publisher));
        } else {
            graph.bind_application_publication(&invariant_installation_receipt);
        }
        let recovered_publication = checkpoint
            .map(|checkpoint| checkpoint.recover_publication(&execution, &graph))
            .transpose()?;
        Ok(WorthQueryPrimaryGraphBootstrap {
            value_bindings: installed_schema.value_bindings().clone(),
            runtime_authority: runtime.authority_identity(),
            installed_packages: runtime.retain_installed_packages(),
            graph,
            resource_support:
                super::super::provider::WorthQueryPrimaryGraphResourceSupport::install(
                    runtime
                        .application_query_resource_profile()
                        .maximum_concurrent_graph_work(),
                    runtime.application_candidate_resource_profile(),
                ),
            product_world_resources,
            rows: Vec::new(),
            external_identities: BTreeSet::new(),
            principal_identities: BTreeSet::new(),
            unique_seed_values: BTreeSet::new(),
            principal_keys: BTreeSet::new(),
            entity_keys: BTreeSet::new(),
            pending_entity_keys: BTreeSet::new(),
            relation_keys: BTreeSet::new(),
            entity_rows: Vec::new(),
            relation_rows: Vec::new(),
            committed_principal_count: 0,
            committed_entity_count: 0,
            committed_relation_count: 0,
            last_seed_commit_id: None,
            seed_batch_failed: false,
            recovered_publication,
            recovered_relational_authority,
            recovered_checkpoint_restore_work,
            mutation_handlers: Default::default(),
            program_activation_seed: None,
            program_activation: None,
            invariant_installation_receipt,
            expected_invariant_inventory_digest,
            _schema: PhantomData,
        })
    }
}

pub(super) fn map_checkpoint_recovery_stop(
    stop: worth_relational::facade::durability::RelationalRecoveryStop,
) -> WorthQueryPrimaryGraphInstallationDenial {
    use worth_relational::facade::durability::RelationalRecoveryStop;
    match stop {
        RelationalRecoveryStop::Durability(error) => primary_graph_denial(
            WorthQueryPrimaryGraphInstallationDenialKind::CheckpointRecoveryRejected,
            format!("native checkpoint recovery denied: {error:?}"),
        ),
        RelationalRecoveryStop::Allocation(cause) => primary_graph_denial(
            WorthQueryPrimaryGraphInstallationDenialKind::NativeAllocation {
                kind: cause.kind(),
                requested_payload_bytes: cause.requested_payload_bytes(),
            },
            "native checkpoint recovery allocation refused",
        ),
        RelationalRecoveryStop::Request(cause) => {
            let translated =
                super::super::provider::relational_execution_denial::relational_execution_kind(
                    cause, None,
                );
            let kind = match translated {
                Ok(kind) => WorthQueryPrimaryGraphInstallationDenialKind::ExecutionDenied { kind },
                Err(kind) => {
                    WorthQueryPrimaryGraphInstallationDenialKind::ExecutionControlStopped { kind }
                }
            };
            primary_graph_denial(
                kind,
                format!("native checkpoint recovery request stopped: {cause:?}"),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn checkpoint_recovery_request_arm_keeps_control_and_memory_by_value() {
        use crate::domain_computation::primary_graph::application_contribution::{
            WorthQueryManagedComputationResourceDenial as Resource,
            WorthQueryMemoryLimitLevel as Level,
        };
        use crate::domain_computation::{
            WorthQueryProviderSessionControlStopKind as Control,
            WorthQueryProviderSessionDenialKind as Denial,
        };
        use worth_relational::facade::{
            durability::RelationalRecoveryStop as Stop,
            transactions::RelationalExecutionDenialCause as Cause,
        };
        use WorthQueryPrimaryGraphInstallationDenialKind as Kind;
        for (cause, kind) in [
            (
                Cause::Cancelled,
                Kind::ExecutionControlStopped {
                    kind: Control::Cancelled,
                },
            ),
            (
                Cause::DeadlineElapsed,
                Kind::ExecutionControlStopped {
                    kind: Control::TimedOut,
                },
            ),
            (
                Cause::PolicyMemoryExhausted {
                    requested: 4096,
                    admitted: 4095,
                    ancestor: 2,
                },
                Kind::ExecutionDenied {
                    kind: Denial::ExecutionResource {
                        denial: Resource::MemoryLimit {
                            requested: 4096,
                            admitted: 4095,
                            level: Level::Policy,
                        },
                        partition_identity: None,
                        policy_ancestor: Some(2),
                    },
                },
            ),
        ] {
            assert_eq!(
                map_checkpoint_recovery_stop(Stop::Request(cause)).kind(),
                kind
            );
        }
    }
    #[test]
    fn checkpoint_recovery_allocation_keeps_physical_kind_and_checked_quote() {
        use worth_execution::{
            ExecutionAllocationDenialKind as AllocationKind, ExecutionAllocationPolicy,
            ExecutionByteBuffer,
        };
        use worth_relational::facade::durability::RelationalRecoveryStop;
        use WorthQueryPrimaryGraphInstallationDenialKind as Kind;

        // Real owner failures: unrepresentable layout has no checked quote;
        // an incomplete, representable backing retains its exact quote.
        let layout =
            ExecutionByteBuffer::allocate(usize::MAX, ExecutionAllocationPolicy::SystemAllocation)
                .unwrap_err();
        let incomplete =
            ExecutionByteBuffer::allocate(17, ExecutionAllocationPolicy::SystemAllocation)
                .unwrap()
                .seal()
                .unwrap_err();
        for (cause, kind, quote) in [
            (layout, AllocationKind::Layout, None),
            (incomplete, AllocationKind::IncompleteSeal, Some(17)),
        ] {
            assert_eq!(
                map_checkpoint_recovery_stop(RelationalRecoveryStop::Allocation(cause)).kind(),
                Kind::NativeAllocation {
                    kind,
                    requested_payload_bytes: quote,
                }
            );
        }
    }
    #[test]
    fn checkpoint_recovery_durability_arm_keeps_the_existing_denial_by_value() {
        use worth_relational::facade::durability::{
            DurabilityError, RecoveryFailureClass, RelationalRecoveryStop,
        };
        let error = DurabilityError::new(
            RecoveryFailureClass::CorruptCheckpoint,
            "invalid checkpoint evidence",
        );
        let expected = primary_graph_denial(
            WorthQueryPrimaryGraphInstallationDenialKind::CheckpointRecoveryRejected,
            format!("native checkpoint recovery denied: {error:?}"),
        );
        assert_eq!(
            map_checkpoint_recovery_stop(RelationalRecoveryStop::Durability(error)),
            expected
        );
    }
}
