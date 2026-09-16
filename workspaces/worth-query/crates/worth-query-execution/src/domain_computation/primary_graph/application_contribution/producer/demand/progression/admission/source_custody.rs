use super::*;

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema + 'static,
{
    pub(in crate::domain_computation::primary_graph) fn retain_required_output_source(
        &self,
        receipt: crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
        change: crate::domain_computation::execution_runtime::product_world::WorthQueryPerformedRelationalProductChange,
        preparation: &crate::domain_computation::primary_graph::application_output_demand::WorthQueryRequiredOutputSourcePreparation,
        root_kind: crate::domain_computation::primary_graph::application_output_demand::PreparedOutputRootKind,
        discovery: Option<std::sync::Arc<dyn std::any::Any + Send + Sync>>,
    ) -> Result<
        (
            crate::domain_computation::primary_graph::WorthQueryPreparedRequiredOutputSource,
            std::sync::Arc<
                crate::domain_computation::primary_graph::WorthQueryApplicationReadObservation,
            >,
        ),
        WorthQueryOutputDemandDenial,
    > {
        let publication = receipt.committed_product_publication();
        let same_runtime =
            std::sync::Arc::ptr_eq(&change.root_identity, &self.product_runtime.root_identity());
        let same_publication = change.product_branch_identity() == publication.product_branch()
            && change.product_commit() == publication.composite_commit();
        if !same_runtime || !same_publication {
            return Err(denial(
                WorthQueryOutputDemandDenialKind::ForeignSource,
                "performed source does not belong to its application publication",
            ));
        }
        let observation = publication
            .take_output_demand_observation()
            .filter(|observation| {
                observation.branch_identity() == change.product_branch_identity()
                    && observation.selected_commit() == change.product_commit()
            })
            .ok_or_else(|| {
                denial(
                    WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable,
                    "performed source publication did not retain its exact output-demand basis",
                )
            })?;
        let retained = crate::domain_computation::primary_graph::WorthQueryApplicationReadObservation::from_product(
            self,
            crate::basis::WorthQueryProductObservationLease::new(observation.clone()),
        );
        let product_occurrence = observation.lifecycle_incarnation();
        let source_commit = self.output_demands.retain_performed_source(
            crate::domain_computation::primary_graph::application_output_demand::WorthQueryPerformedOutputDemandSource {
                receipt,
                change: std::sync::Arc::new(change),
                observation,
                output_source_identity: None,
            },
            preparation,
            root_kind,
            discovery,
        )?;
        Ok((
            crate::domain_computation::primary_graph::WorthQueryPreparedRequiredOutputSource {
                runtime_authority: self.runtime.authority_identity().as_u64(),
                source_commit,
                product_occurrence,
                owner: self.output_demands.clone(),
            },
            retained,
        ))
    }

    pub(in crate::domain_computation::primary_graph) fn recover_discovered_output_source<
        Discovery,
    >(
        &self,
        receipt: &crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
        root_type: std::any::TypeId,
    ) -> Result<
        (
            Discovery,
            std::sync::Arc<
                crate::domain_computation::primary_graph::WorthQueryApplicationReadObservation,
            >,
            crate::domain_computation::primary_graph::WorthQueryPreparedRequiredOutputSource,
        ),
        WorthQueryOutputDemandDenial,
    >
    where
        Discovery: Clone + Send + Sync + 'static,
    {
        let (discovery, observation) = self
            .output_demands
            .recover_discovered_source::<Discovery>(receipt, root_type)?;
        let retained = crate::domain_computation::primary_graph::WorthQueryApplicationReadObservation::from_product(
            self,
            crate::basis::WorthQueryProductObservationLease::new(observation),
        );
        let prepared =
            crate::domain_computation::primary_graph::WorthQueryPreparedRequiredOutputSource {
                runtime_authority: self.runtime.authority_identity().as_u64(),
                source_commit: receipt
                    .committed_product_publication()
                    .composite_commit()
                    .clone(),
                product_occurrence: receipt.product_branch().occurrence(),
                owner: self.output_demands.clone(),
            };
        Ok((discovery, retained, prepared))
    }

    pub(in crate::domain_computation::primary_graph) fn bind_prepared_output_source<
        Query,
        Value,
    >(
        &self,
        prepared: &crate::domain_computation::primary_graph::WorthQueryPreparedRequiredOutputSource,
        source: &crate::domain_computation::primary_graph::WorthQueryApplicationOutputDemandSource<
            Query,
            Value,
        >,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        let observed = source
            .observed_sources()
            .first()
            .filter(|_| source.rows().len() == 1 && source.observed_sources().len() == 1);
        let Some(observed) = observed else {
            return Err(denial(
                WorthQueryOutputDemandDenialKind::ForeignSource,
                "prepared output source query did not return one owner-paired occurrence",
            ));
        };
        validate_prepared_source_carrier(
            self.runtime.authority_identity().as_u64(),
            prepared,
            observed,
        )?;
        self.output_demands.bind_prepared_output_source(
            &prepared.source_commit,
            crate::domain_computation::primary_graph::application_output_demand::BoundOutputSource {
                scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(observed.footprint.root),
                identity: observed.idempotency_identity(),
            },
        )
    }

    pub(in crate::domain_computation::primary_graph) fn bind_prepared_output_sources<
        Query,
        Value,
    >(
        &self,
        prepared: &crate::domain_computation::primary_graph::WorthQueryPreparedRequiredOutputSource,
        sources: &[crate::domain_computation::primary_graph::WorthQueryApplicationOutputDemandSource<
            Query,
            Value,
        >],
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        let identities = sources
            .iter()
            .map(|source| {
                let observed = source
                    .observed_sources()
                    .first()
                    .filter(|_| source.rows().len() == 1 && source.observed_sources().len() == 1)
                    .ok_or_else(|| {
                        denial(
                            WorthQueryOutputDemandDenialKind::ForeignSource,
                            "discovered root source did not return one owner-paired occurrence",
                        )
                    })?;
                validate_prepared_source_carrier(
                    self.runtime.authority_identity().as_u64(),
                    prepared,
                    observed,
                )?;
                Ok(crate::domain_computation::primary_graph::application_output_demand::BoundOutputSource {
                    scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(observed.footprint.root),
                    identity: observed.idempotency_identity(),
                })
            })
            .collect::<Result<Vec<_>, WorthQueryOutputDemandDenial>>()?;
        self.output_demands
            .ensure_discovered_sources_bound(&prepared.source_commit, &identities)
    }

    pub fn discard_prepared_required_output_source(
        &self,
        prepared: crate::domain_computation::primary_graph::WorthQueryPreparedRequiredOutputSource,
    ) {
        if prepared.runtime_authority == self.runtime.authority_identity().as_u64() {
            self.output_demands
                .discard_prepared_source(&prepared.source_commit);
        }
    }
}
