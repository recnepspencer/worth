use super::super::application_output_demand::AcceptedCheckpointFactSource;
use super::{
    facts, WorthQueryApplicationCheckpoint, WorthQueryApplicationCheckpointSectionBytes,
    WorthQueryCheckpointCaptureDenial, WorthQueryCheckpointCapturePolicy,
};

mod native_priors;
mod output_facts;

pub(super) fn merge_accepted_outputs(
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
    #[cfg(feature = "test-query-execution-observer")]
    #[doc(hidden)]
    pub fn capture_native_truth_checkpoint_for_test(
        &self,
    ) -> Result<WorthQueryApplicationCheckpoint, WorthQueryCheckpointCaptureDenial> {
        let policy = WorthQueryCheckpointCapturePolicy::SystemAllocation;
        self.primary_provider.graph.with_runtime(|runtime| {
            runtime
                .durability_authority()
                .native_checkpoint(policy)
                .map_err(WorthQueryCheckpointCaptureDenial::from)
                .and_then(|native| {
                    WorthQueryApplicationCheckpoint::encode(native, self.publication(), &[], policy)
                        .map(|(checkpoint, _)| checkpoint)
                })
        })
    }

    pub fn capture_application_checkpoint(
        &self,
        policy: WorthQueryCheckpointCapturePolicy<'_, '_>,
    ) -> Result<WorthQueryApplicationCheckpoint, WorthQueryCheckpointCaptureDenial> {
        self.capture_application_checkpoint_with_sections(policy)
            .map(|(checkpoint, _)| checkpoint)
    }

    /// Capture one checkpoint and its encoder-owned section sizes together.
    /// The sizes describe these exact bytes; no second capture is performed.
    pub fn capture_application_checkpoint_with_sections(
        &self,
        policy: WorthQueryCheckpointCapturePolicy<'_, '_>,
    ) -> Result<
        (
            WorthQueryApplicationCheckpoint,
            WorthQueryApplicationCheckpointSectionBytes,
        ),
        WorthQueryCheckpointCaptureDenial,
    > {
        policy.check_live()?;
        let occurrence = self.product_runtime.default_occurrence;
        let lane = self
            .primary_provider
            .application_branch_commit_lane_for_occurrence(occurrence)
            .map_err(|_| {
                native_priors::capture_denial("checkpoint branch coordination is unavailable")
            })?;
        let _coordination = lane.enter();
        let lease = self
            .product_runtime
            .admit_product_occurrence(occurrence)
            .map_err(|_| {
                native_priors::capture_denial("checkpoint product occurrence cannot be admitted")
            })?;
        self.primary_provider.graph.with_runtime(|runtime| {
            policy.check_live()?;
            runtime
                .durability_authority()
                .native_checkpoint(policy)
                .map_err(WorthQueryCheckpointCaptureDenial::from)
                .and_then(|checkpoint| {
                    let mut accepted = self.output_demands.accepted_checkpoint_records();
                    let mut admission = self
                        .primary_provider
                        .graph
                        .source_owner
                        .invalidation_owner
                        .edit_admission();
                    let lineage = self
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
                            self.runtime.authority_identity().as_u64(),
                            &self.installed_schema.binding_identity(),
                            occurrence,
                            lease.observation().reference_generation().get(),
                            &mut admission,
                        )
                        .map_err(|_| {
                            native_priors::capture_denial(
                                "checkpoint native output heads cannot be selected",
                            )
                        })?;
                    for (identity, source, _) in &mut accepted {
                        policy.check_live()?;
                        let Some(source) = source else {
                            continue;
                        };
                        // A mismatch or unsupported fact kind leaves no reusable
                        // payload. Neither the query footprint nor the digest can
                        // reconstruct a producer's original decision reads.
                        identity.producer_facts = match source {
                            AcceptedCheckpointFactSource::Committed(receipt) => lineage
                                .checkpoint_facts_for_receipt(receipt)
                                .and_then(|(facts, witness)| {
                                    output_facts::encode(&facts, witness, &mut admission)
                                }),
                            AcceptedCheckpointFactSource::Stable(stable) => stable
                                .checkpoint_source_facts()
                                .and_then(|facts| facts::encode(&facts)),
                        };
                        identity.producer_fact_wire_version = if identity.producer_facts.is_some() {
                            facts::WIRE_VERSION
                        } else {
                            0
                        };
                    }
                    let accepted_outputs = native_priors::merge(
                        &lineage,
                        accepted
                            .into_iter()
                            .map(|(identity, _, binding)| (identity, binding)),
                        self.recovered_outputs.iter().map(|accepted| {
                            (
                                accepted.checkpoint.clone(),
                                accepted.correspondence.binding_type(),
                            )
                        }),
                        priors,
                    )
                    .map_err(|_| {
                        native_priors::capture_denial(
                            "checkpoint output family binding is ambiguous",
                        )
                    })?;
                    drop(lineage);
                    WorthQueryApplicationCheckpoint::encode(
                        checkpoint,
                        self.publication(),
                        &accepted_outputs,
                        policy,
                    )
                })
        })
    }
}
