//! Fresh admission and consuming handoff for one retained original publication.
use super::*;
use crate::domain_computation::primary_graph::application_installation::WorthQueryRecoveredProgramOutputSource;
impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub(super) fn recover_admitted_unpublished_application_delivery<Operation, Input, Scope>(
        &self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        recovery: &WorthQueryProductUnpublishedRecovery,
        admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
        mut delivery: RecoveredPublicationDelivery<'_>,
    ) -> Result<
        (
            WorthQueryManagedApplicationRecoveryOutcome,
            Option<WorthQueryRecoveredProgramOutputSource>,
        ),
        WorthQueryManagedApplicationRecoveryDenial,
    >
    where
        Input: Clone + Send + Sync + 'static,
    {
        use WorthQueryManagedApplicationRecoveryDenial as Denial;
        let request = phase
            .execution_request_for(&self.runtime.product_runtime)
            .map_err(|cause| Denial::ExecutionDenied(cause.into()))?;
        {
            if !admission.belongs_to(
                self.runtime.authority_identity(),
                &self.installed_schema.binding_identity(),
            ) {
                return Err(Denial::ForeignAdmission);
            }
            admission
                .validate_installed_operation(&self.runtime.installed_operations)
                .and_then(|()| admission.validate_current_authority())
                .map_err(Denial::Authorization)?;
            let product = admission
                .graph_work()
                .mutation_product()
                .ok_or(Denial::ForeignAdmission)?
                .publication_binding();
            let bound = idempotency
                .bind_operation(admission.operation_definition_identity())
                .bind_operation_scope(admission.operation_scope_binding())
                .bind_preconditions(admission.mutation_preconditions().identity())
                .bind_governed_input(admission.governed_input_identity())
                .bind_governed_proposal(admission.governed_proposal_identity());
            let command_lane = self.runtime.primary_provider.correction_commit_lane(&bound);
            let command_coordination = command_lane.as_ref().map(|lane| lane.enter());
            let lane = self
                .runtime
                .primary_provider
                .application_branch_commit_lane(product.observation());
            let coordination = lane.enter();
            let mut guard = self
                .runtime
                .primary_provider
                .begin_managed_unpublished_recovery(recovery.record_handle(), product.recovery())
                .map_err(Denial::from)?;
            // The handler already supplied this component to the retained attempt.
            // Fresh authority, scope, input and preconditions still compare in full.
            let definition_identity = guard.attempt_mut().conditional_definition_identity();
            let idempotency = if idempotency.conditional_definition_identity().is_none() {
                idempotency.bind_conditional_definition(definition_identity.as_ref())
            } else {
                idempotency
            };
            let bound = if bound.conditional_definition_identity().is_none() {
                bound.bind_conditional_definition(definition_identity.as_ref())
            } else {
                bound
            };
            if !guard.attempt_mut().matches_fresh_binding(
                bound,
                admission.operation_scope_binding(),
                product.observation(),
            ) {
                return Err(Denial::BindingMismatch);
            }
            let producer_memory = producer::reserve_original_output(&request, &delivery)?;
            let needs_settlement = recovery
                .inspect()
                .map_err(Denial::World)?
                .relational_requires_settlement();
            if needs_settlement {
                recovery
                    .continue_owner_settlement()
                    .map_err(Denial::World)?;
            }
            let unpublished = recovery.inspect().map_err(Denial::World)?;
            let lease = self
                .runtime
                .product_runtime
                .admit_product_occurrence(product.observation().lifecycle_incarnation())
                .map_err(|_| Denial::ProductUnavailable)?;
            let binding = lease.publication_binding();
            if unpublished.expected_product() != binding.observation() {
                return Err(Denial::BindingMismatch);
            }
            let outcome = publication::publish(
                self,
                request,
                admission.publication_request(),
                &binding,
                recovery,
                unpublished,
                &mut guard,
            )?;
            drop(binding);
            drop(lease);
            match outcome {
                Err(PublicationDenial::ClockMismatch) => {
                    Err(Denial::ExecutionDenied(crate::domain_computation::primary_graph::WorthQueryAdvancementDenial::Resource(crate::domain_computation::primary_graph::WorthQueryManagedComputationResourceDenial::ForeignAdmissionClock)))
                },
Err(PublicationDenial::Stopped(stop)) => {
                    Err(Denial::ExecutionDenied(advancement_denial(stop)))
                }
                Err(PublicationDenial::Refused(_)) => {
                    Ok((WorthQueryManagedApplicationRecoveryOutcome::NoEffect, None))
                }
                Ok(PublicationOutcome::ProductUnpublished(effects)) => {
                    let next = effects.into_recovery();
                    guard.retain_successor(next.record_handle());
                    let prior_cleanup_failure = self
                        .release_product_publication_recovery_in_advancement(
                            phase,
                            recovery.clone(),
                            0,
                        )
                        .err();
                    Ok((
                        WorthQueryManagedApplicationRecoveryOutcome::ProductUnpublished {
                            next,
                            prior_cleanup_failure,
                        },
                        None,
                    ))
                }
                Ok(PublicationOutcome::Performed((publication, definition_generation))) => {
                    let original_owner = publication
                        .new_product_head()
                        .branch_identity()
                        .owner_identity();
                    let original_world_commit =
                        publication.new_product_head().selected_commit().ordinal();
                    let native = publication
                        .new_product_head()
                        .basis()
                        .relational_basis()
                        .reference()
                        .target()
                        .as_basis()
                        .expect("actual performed Native successor");
                    let original_native_commit = native.selected_commit_id();
                    let original_native_version = native.version_id();
                    let session = guard
                        .take_attempt()
                        .complete(publication, definition_generation);
                    let mut source = match &mut delivery {
                        RecoveredPublicationDelivery::Ordinary
                        | RecoveredPublicationDelivery::OwningSource => None,
                        RecoveredPublicationDelivery::ProgramSource | RecoveredPublicationDelivery::Producer(_) => {
                            super::super::recovered_program_source_delivery::consume_first_if_armed(
                                self,
                                || session.take_recovered_output_source(),
                            );
                            Some(session.take_recovered_output_source())
                        }
                    };
                    let ordinary = matches!(&delivery, RecoveredPublicationDelivery::Ordinary);
                    let producer = matches!(&delivery, RecoveredPublicationDelivery::Producer(_));
                    let publication_result = match delivery {
                        RecoveredPublicationDelivery::Producer(retain) => publish_recovered(
                            &request, &self.runtime.primary_provider, session, producer_memory,
                            |facts| {
                                retain(ProducerRecoveryCustody {
                                    facts: facts.expect("producer recovery reserved original evidence"),
                                    carrier: source.take(), binding: bound, cleanup: Some(recovery.clone()),
                                });
                                // The producer continuation now owns the performed bundle.
                                // Do not restore an empty unpublished entry on unwind.
                                guard.finish();
                            }),
                        RecoveredPublicationDelivery::Ordinary | RecoveredPublicationDelivery::ProgramSource | RecoveredPublicationDelivery::OwningSource => {
                            let result = publish_recovered(&request, &self.runtime.primary_provider, session, producer_memory, |_| {});
                            guard.finish();
                            result
                        }
                    };
                    let publication_failure = publication_result.err();
                    let (prior_cleanup_failure, pending_cleanup) = if ordinary {
                        (self.release_product_publication_recovery_in_advancement(phase, recovery.clone(), 0).err(), None)
                    } else if producer { (None, None) } else { (None, Some(recovery.clone())) };
                    drop(coordination);
                    // The installed pending/completed handoff now owns visibility.
                    drop(command_coordination);
                    let read = self.resolve_admitted_application_idempotency(
                        phase,
                        admission,
                        idempotency,
                    );
                    Ok((
                        WorthQueryManagedApplicationRecoveryOutcome::Performed(
                            WorthQueryManagedApplicationRecoveryPerformed {
                                original_owner,
                                original_world_commit,
                                original_native_commit,
                                original_native_version,
                                original_key: *bound.key_identity(),
                                conditional_definition_identity: definition_identity,
                                read,
                                publication_failure,
                                prior_cleanup_failure,
                                pending_cleanup,
                            },
                        ),
                        source,
                    ))
                }
            }
        }
    }
}
