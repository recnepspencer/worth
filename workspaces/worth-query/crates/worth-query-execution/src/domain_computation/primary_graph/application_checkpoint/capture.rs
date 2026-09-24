use super::{facts, WorthQueryApplicationCheckpoint, WorthQueryApplicationCheckpointSectionBytes};

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
    pub fn capture_application_checkpoint(
        &self,
    ) -> Result<
        WorthQueryApplicationCheckpoint,
        worth_relational::facade::durability::DurabilityError,
    > {
        self.capture_application_checkpoint_with_sections()
            .map(|(checkpoint, _)| checkpoint)
    }

    /// Capture one checkpoint and its encoder-owned section sizes together.
    /// The sizes describe these exact bytes; no second capture is performed.
    pub fn capture_application_checkpoint_with_sections(
        &self,
    ) -> Result<
        (
            WorthQueryApplicationCheckpoint,
            WorthQueryApplicationCheckpointSectionBytes,
        ),
        worth_relational::facade::durability::DurabilityError,
    > {
        self.primary_provider.graph.with_runtime(|runtime| {
            runtime
                .durability_authority()
                .native_checkpoint()
                .map(|checkpoint| {
                    let mut accepted = self.output_demands.accepted_checkpoint_records();
                    let lineage = self
                        .primary_provider
                        .graph
                        .output_lineage
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                    for (identity, receipt) in &mut accepted {
                        let Some(receipt) = receipt else {
                            continue;
                        };
                        // A mismatch or unsupported fact kind leaves no reusable
                        // payload. Neither the query footprint nor the digest can
                        // reconstruct a producer's original decision reads.
                        identity.producer_facts = lineage
                            .producer_facts_for_receipt(receipt)
                            .and_then(|facts| facts::encode(&facts));
                    }
                    drop(lineage);
                    let accepted_outputs = merge_accepted_outputs(
                        accepted.into_iter().map(|(identity, _)| identity).collect(),
                        self.recovered_outputs
                            .iter()
                            .map(|accepted| accepted.checkpoint.clone()),
                    );
                    WorthQueryApplicationCheckpoint::encode(
                        checkpoint,
                        self.publication(),
                        &accepted_outputs,
                    )
                })
        })
    }
}
