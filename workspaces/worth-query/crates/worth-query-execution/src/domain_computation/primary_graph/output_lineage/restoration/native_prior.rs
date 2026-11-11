//! Restore a producer antecedent as prior custody, without claiming a producer run.
use super::*;
use crate::domain_computation::primary_graph::application_checkpoint::CheckpointDecodeDenial;
impl WorthQueryApplicationOutputLineage {
    pub(in crate::domain_computation::primary_graph) fn record_native_restored_prior(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime_authority: u64,
        schema: &ApplicationSchemaBindingIdentity,
        observation: &worth_runtime_world::facade::ProductBranchObservation,
        prior: worth_relational::facade::mvcc::RelationalRestoredProducerPrior,
    ) -> Result<(), CheckpointDecodeDenial> {
        let publication = prior.publication();
        let meaning = publication.meaning();
        let original_scope = crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(prior.producer_scope());
        let bindings = self
            .output_families
            .get(meaning.family.as_str())
            .ok_or_else(|| {
                CheckpointDecodeDenial::from("restored producer family is not installed".to_owned())
            })?
            .iter()
            .map(|(binding, _)| *binding)
            .collect::<Vec<_>>();
        let candidates = self.retained_output_candidates_metered(runtime_authority, schema, original_scope,
            observation.lifecycle_incarnation(), observation.reference_generation().get(), &bindings, *meaning.source_partition.bytes(), execution)
            .map_err(|stop| match stop {
                worth_relational::facade::mvcc::CompanionPreflightStop::Execution(cause) => CheckpointDecodeDenial::from_fact(
                    crate::domain_computation::primary_graph::application_checkpoint::facts::FactDecodeDenial::Execution(cause)),
                _ => CheckpointDecodeDenial::from(format!("restored producer antecedent unavailable: {stop:?}")),
            })?;
        let mut antecedent = None;
        for candidate in candidates {
            execution
                .consult()
                .map_err(CheckpointDecodeDenial::execution)?;
            let pin = &candidate.settlement_identity;
            let coordinate = pin.coordinate();
            let recorded = self
                .by_source
                .get(pin.source())
                .and_then(|rows| rows.get(&coordinate.occurrence))
                .and_then(|rows| rows.get(&coordinate.generation))
                .and_then(|rows| rows.get(pin.slot()))
                .and_then(|row| row.get())
                .expect("retained antecedent candidate owns its exact row");
            let matches = recorded
                .required_publication
                .as_ref()
                .is_some_and(|previous| prior.matches_producer_publication(previous))
                || recorded
                    .native_prior_checkpoint
                    .as_ref()
                    .is_some_and(|prior| {
                        prior.producer() == publication.producer()
                            && prior.source() == *meaning.source_epoch.bytes()
                    });
            if matches {
                if antecedent.is_some() {
                    return Err("restored producer antecedent is ambiguous"
                        .to_owned()
                        .into());
                }
                antecedent = Some((
                    candidate.correspondence,
                    recorded.producer_dependency_identity,
                    recorded.idempotency_key_identity,
                ));
            }
        }
        let (prior, dependency, key) = antecedent.ok_or_else(|| {
            CheckpointDecodeDenial::from(
                "restored publication has no retained producer antecedent".to_owned(),
            )
        })?;
        let correspondence = Arc::new(prior.transport_native_restoration(publication, execution)?);
        let binding = correspondence.binding_type().ok_or_else(|| {
            CheckpointDecodeDenial::from("restored producer antecedent has no binding".to_owned())
        })?;
        let scope = crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(meaning.source_scope);
        let partition = *meaning.source_partition.bytes();
        self.record_prior_output(
            binding,
            runtime_authority,
            schema.clone(),
            scope,
            observation.lifecycle_incarnation(),
            observation.reference_generation().get(),
            correspondence,
            PriorOutputOrigin::NativeRestored(publication.clone()),
            partition,
            dependency,
            key,
        );
        Ok(())
    }
}
