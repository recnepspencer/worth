use super::super::application_output_demand::AcceptedCheckpointFactSource;
use super::{
    facts, WorthQueryApplicationCheckpoint, WorthQueryApplicationCheckpointSectionBytes,
    WorthQueryCheckpointCaptureDenial, WorthQueryCheckpointCapturePolicy,
};

pub(crate) mod native_priors;
pub(in crate::domain_computation::primary_graph) mod output_facts;

pub fn merge_accepted_outputs(
    mut current: Vec<
        super::super::application_output_demand::WorthQueryAcceptedOutputCheckpointIdentity,
    >,
    recovered: impl IntoIterator<
        Item = super::super::application_output_demand::WorthQueryAcceptedOutputCheckpointIdentity,
    >,
) -> Vec<super::super::application_output_demand::WorthQueryAcceptedOutputCheckpointIdentity> {
    let unshadowed = recovered
        .into_iter()
        .filter(|recovered| {
            !current
                .iter()
                .any(|accepted| accepted.same_output_slot(recovered))
        })
        .collect::<Vec<_>>();
    current.extend(unshadowed);
    current.sort_by(|left, right| left.canonical_cmp(right));
    current.dedup();
    current
}

impl<Schema> super::super::WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: worth_query_installation::facade::ApplicationSchema + 'static,
{
    /// Capture the same native truth with no derived Query records, so the
    /// certification reference must execute a fresh producer computation.
    #[doc(hidden)]
    pub fn capture_native_truth_checkpoint_for_test(
        &self,
        phase: &super::super::WorthQueryAdvancementPhase<'_>,
    ) -> Result<WorthQueryApplicationCheckpoint, WorthQueryCheckpointCaptureDenial> {
        self.capture_checkpoint_sections(
            phase,
            WorthQueryCheckpointCapturePolicy::SystemAllocation,
            false,
        )
        .map(|(checkpoint, _)| checkpoint)
    }

    pub fn capture_application_checkpoint(
        &self,
        phase: &super::super::WorthQueryAdvancementPhase<'_>,
        policy: WorthQueryCheckpointCapturePolicy<'_, '_>,
    ) -> Result<WorthQueryApplicationCheckpoint, WorthQueryCheckpointCaptureDenial> {
        self.capture_application_checkpoint_with_sections(phase, policy)
            .map(|(checkpoint, _)| checkpoint)
    }

    /// Capture one checkpoint and its encoder-owned section sizes together.
    /// The sizes describe these exact bytes; no second capture is performed.
    pub fn capture_application_checkpoint_with_sections(
        &self,
        phase: &super::super::WorthQueryAdvancementPhase<'_>,
        policy: WorthQueryCheckpointCapturePolicy<'_, '_>,
    ) -> Result<
        (
            WorthQueryApplicationCheckpoint,
            WorthQueryApplicationCheckpointSectionBytes,
        ),
        WorthQueryCheckpointCaptureDenial,
    > {
        self.capture_checkpoint_sections(phase, policy, true)
    }

    fn capture_checkpoint_sections(
        &self,
        phase: &super::super::WorthQueryAdvancementPhase<'_>,
        policy: WorthQueryCheckpointCapturePolicy<'_, '_>,
        with_outputs: bool,
    ) -> Result<
        (
            WorthQueryApplicationCheckpoint,
            WorthQueryApplicationCheckpointSectionBytes,
        ),
        WorthQueryCheckpointCaptureDenial,
    > {
        policy.check_live()?;
        let occurrence = self.runtime.product_runtime.default_occurrence;
        let lane = self
            .runtime
            .primary_provider
            .application_branch_commit_lane_for_occurrence(occurrence);
        let _coordination = lane.enter();
        let lease = self
            .runtime
            .product_runtime
            .admit_product_occurrence(occurrence)
            .map_err(|_| {
                native_priors::capture_denial("checkpoint product occurrence cannot be admitted")
            })?;
        let execution = phase.execution_request_for(&self.runtime.product_runtime)?;
        let execution = match policy {
            WorthQueryCheckpointCapturePolicy::SystemAllocation => execution,
            WorthQueryCheckpointCapturePolicy::Execution(lease) => {
                worth_execution::ExecutionRequest::leased(lease)
            }
        };
        let mut answer = None;
        let stage = execution.run_native_with_checkpoint(|_| {
            answer = Some(self.runtime.primary_provider.graph.with_runtime(
                &execution,
                |runtime| {
                    policy.check_live()?;
                    runtime
                        .durability_authority()
                        .capture_interim_checkpoint(&execution, policy)
                        .map_err(WorthQueryCheckpointCaptureDenial::from)
                        .and_then(|(checkpoint, admission)| {
                            let port = self
                                .runtime
                                .product_runtime
                                .interim_checkpoint_port()
                                .ok_or(
                                    WorthQueryCheckpointCaptureDenial::OriginalAuthorityUnavailable,
                                )?;
                            let world = port
                                .capture(
                                    &execution,
                                    lease.observation(),
                                    &admission,
                                    super::interim_transport::application_binding(
                                        self.publication(),
                                    ),
                                )
                                .map_err(|cause| {
                                    WorthQueryCheckpointCaptureDenial::WorldOriginals(Box::new(
                                        cause,
                                    ))
                                })?;
                            if !with_outputs {
                                return WorthQueryApplicationCheckpoint::encode_captured(
                                    checkpoint,
                                    admission,
                                    world,
                                    self.publication(),
                                    &[],
                                    &execution,
                                    policy,
                                );
                            }
                            let accepted =
                                self.runtime.output_demands.accepted_checkpoint_records();

                            let lineage = self
                                .runtime
                                .primary_provider
                                .graph
                                .output_lineage
                                .lock()
                                .unwrap_or_else(std::sync::PoisonError::into_inner);
                            // Native prior custody is required; cached reuse facts are
                            // best effort. Admit the locators before optional encoding
                            // can consume this checkpoint's remaining allowance.
                            let priors = lineage
                                .checkpoint_prior_outputs(
                                    &execution,
                                    self.runtime.authority_identity().as_u64(),
                                    &self.installed_schema.binding_identity(),
                                    occurrence,
                                    lease.observation().reference_generation().get(),
                                )
                                .map_err(|denial| denial.into_capture_denial())?;
                            let mut accepted = native_priors::merge(
                                &lineage,
                                accepted,
                                self.runtime.recovered_outputs.iter().map(|accepted| {
                                    (
                                        accepted.checkpoint.clone(),
                                        accepted.correspondence.binding_type(),
                                    )
                                }),
                                priors,
                            )?;
                            for (identity, source) in &mut accepted {
                                policy.check_live()?;
                                let Some(source) = source else {
                                    continue;
                                };
                                // A mismatch or unsupported fact kind leaves no reusable
                                // payload. Neither the query footprint nor the digest can
                                // reconstruct a producer's original decision reads.
                                identity.producer_facts = match source {
                                    AcceptedCheckpointFactSource::Committed(receipt) => lineage
                                        .checkpoint_facts_for_publication((&*receipt).into())
                                        .and_then(|(facts, witness)| {
                                            output_facts::encode(&facts, witness)
                                        }),
                                    AcceptedCheckpointFactSource::Recovered(publication) => lineage
                                        .checkpoint_facts_for_publication(publication.view())
                                        .and_then(|(facts, witness)| {
                                            output_facts::encode(&facts, witness)
                                        }),
                                    AcceptedCheckpointFactSource::Stable(stable) => {
                                        stable.checkpoint_source_facts().and_then(|facts| {
                                            match stable.checkpoint_output_witness() {
                                                Some(witness) => {
                                                    output_facts::encode(&facts, witness)
                                                }
                                                // Existing authenticated checkpoint facts can
                                                // already carry their original output revisions.
                                                None => facts::encode(&facts),
                                            }
                                        })
                                    }
                                }
                                .map(std::sync::Arc::new);
                                identity.producer_fact_wire_version =
                                    if identity.producer_facts.is_some() {
                                        facts::WIRE_VERSION
                                    } else {
                                        0
                                    };
                            }
                            let mut accepted_outputs = accepted
                                .into_iter()
                                .map(|(identity, _)| identity)
                                .collect::<Vec<_>>();
                            accepted_outputs.sort_by(|left, right| left.canonical_cmp(right));
                            accepted_outputs.dedup();
                            drop(lineage);
                            WorthQueryApplicationCheckpoint::encode_captured(
                                checkpoint,
                                admission,
                                world,
                                self.publication(),
                                &accepted_outputs,
                                &execution,
                                policy,
                            )
                        })
                },
            ));
        });
        match (stage, answer) {
            (Ok(_), Some(answer)) => answer,
            (Ok(_), None) => unreachable!("the checkpoint stage produced its answer"),
            (Err(cause), Some(Err(inner))) if worth_relational::facade::transactions::RelationalExecutionDenialCause::from(cause)
                == worth_relational::facade::transactions::RelationalExecutionDenialCause::NestedStopped => Err(inner),
            (Err(cause), _) => Err(WorthQueryCheckpointCaptureDenial::from(
                worth_relational::facade::durability::RelationalNativeCheckpointCaptureDenial::Request(cause.into()))),
        }
    }
}
