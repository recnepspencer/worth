use super::*;

impl WorthQueryApplicationOutputLineage {
    #[allow(clippy::too_many_arguments)]
    pub(in crate::domain_computation::primary_graph) fn record_restoration(
        &mut self,
        output_binding: TypeId,
        runtime_authority: u64,
        schema: ApplicationSchemaBindingIdentity,
        scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
        observation: &worth_runtime_world::facade::ProductBranchObservation,
        correspondence: Arc<WorthQueryApplicationOutputCorrespondence>,
        source_identity: [u8; 32],
        source_partition_identity: [u8; 32],
        producer_dependency_identity: Option<[u8; 32]>,
        idempotency_key_identity: [u8; 32],
        observed_source_facts: Arc<[crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationObservedFact]>,
    ) {
        let source = SemanticSource {
            runtime_authority,
            schema,
            scope,
            output_binding,
        };
        let replaced = self
            .by_source
            .entry(source)
            .or_default()
            .entry(observation.lifecycle_incarnation())
            .or_default()
            .insert(
                observation.reference_generation().get(),
                RecordedOutput {
                    correspondence,
                    source_identity: Some(source_identity),
                    source_partition_identity: Some(source_partition_identity),
                    producer_dependency_identity,
                    idempotency_key_identity,
                    observed_source_facts,
                },
            );
        assert!(
            replaced.is_none(),
            "one product generation may restore one output binding once"
        );
        self.live_occurrences
            .insert(observation.lifecycle_incarnation());
    }

    pub(in crate::domain_computation::primary_graph) fn install_output_families(
        &mut self,
        families: BTreeMap<String, Vec<TypeId>>,
    ) {
        assert!(self.output_families.is_empty());
        self.output_families.extend(families);
    }

    pub(crate) fn register_fork(
        &mut self,
        source: &worth_runtime_world::facade::ProductBranchObservation,
        destination: &worth_runtime_world::facade::ProductBranchObservation,
    ) {
        let source = ProductCoordinate {
            occurrence: source.lifecycle_incarnation(),
            generation: source.reference_generation().get(),
        };
        let destination = destination.lifecycle_incarnation();
        self.live_occurrences.insert(source.occurrence);
        self.live_occurrences.insert(destination);
        assert!(
            self.origins.insert(destination, source).is_none(),
            "one product occurrence may be registered as a fork once"
        );
    }

    pub(in crate::domain_computation::primary_graph) fn record(
        &mut self,
        application: &WorthQueryPrimaryGraphCommittedApplication,
    ) {
        let evidence = application.commit_evidence();
        let correspondence = evidence.output_correspondence();
        let scope = evidence.operation_scope();
        let head = application.product_publication().new_product_head();
        let coordinate = ProductCoordinate {
            occurrence: head.lifecycle_incarnation(),
            generation: head.reference_generation().get(),
        };
        self.live_occurrences.insert(coordinate.occurrence);
        let Some(output_binding) = correspondence.binding_type() else {
            return;
        };
        let source = SemanticSource {
            runtime_authority: scope.runtime_authority(),
            schema: scope.binding_identity().clone(),
            scope: scope.scope(),
            output_binding,
        };
        let replaced = self
            .by_source
            .entry(source)
            .or_default()
            .entry(head.lifecycle_incarnation())
            .or_default()
            .insert(
                head.reference_generation().get(),
                RecordedOutput {
                    correspondence: evidence.retain_output_correspondence(),
                    source_identity: evidence.idempotency().source_identity(),
                    source_partition_identity: evidence.idempotency().source_partition_identity(),
                    producer_dependency_identity: evidence
                        .idempotency()
                        .producer_dependency_identity(),
                    idempotency_key_identity: *evidence.idempotency().key_identity(),
                    observed_source_facts: evidence.retain_observed_source_facts(),
                },
            );
        assert!(
            replaced.is_none(),
            "one product generation may publish one output binding once"
        );
    }
}
