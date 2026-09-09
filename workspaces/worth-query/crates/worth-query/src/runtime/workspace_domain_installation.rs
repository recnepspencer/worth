use super::WorthQueryWorkspace;

impl WorthQueryWorkspace {
    pub(crate) fn query_execution_runtime(
        &self,
    ) -> &worth_query_execution::facade::runtime::WorthQueryExecutionRuntime {
        self.runtime.query_execution_runtime()
    }

    pub(crate) fn operating_world<L: crate::basis_lifecycle::BasisOperationLane>(
        &self,
        entry: crate::domain_installation::WorthQueryOperatingWorldEntry<L>,
    ) -> crate::domain_installation::WorthQueryInstalledOperatingWorld<'_, L> {
        crate::domain_installation::WorthQueryInstalledOperatingWorld::new(
            &self.runtime,
            crate::domain_installation::WorthQueryOperatingWorldBasis::Component(
                entry.into_capability(),
            ),
        )
    }

    pub fn observe_operating_world(
        &self,
    ) -> Result<
        crate::domain_installation::WorthQueryInstalledOperatingWorld<
            '_,
            crate::basis_lifecycle::ObservationLaneWitness,
        >,
        crate::domain_installation::WorthQueryOperatingWorldEntryDenial,
    > {
        if let Some(installed) = &self.runtime.installed_product {
            let product = installed
                .world
                .admit_product_branch(installed.world.default_branch())
                .map_err(product_admission_denial)?;
            let entry =
                crate::domain_installation::WorthQueryOperatingWorldEntry::observe_current()?;
            return Ok(self.product_operating_world(entry, product));
        }
        crate::domain_installation::WorthQueryOperatingWorldEntry::observe_current()
            .map(|entry| self.operating_world(entry))
    }

    pub fn observe_branch_operating_world(
        &self,
        branch_identity: crate::domain_installation::WorthQueryBranchHeadIdentity,
    ) -> Result<
        crate::domain_installation::WorthQueryInstalledOperatingWorld<
            '_,
            crate::basis_lifecycle::ObservationLaneWitness,
        >,
        crate::domain_installation::WorthQueryOperatingWorldEntryDenial,
    > {
        self.require_component_operating_world()?;
        crate::domain_installation::WorthQueryOperatingWorldEntry::observe_branch(&branch_identity)
            .map(|entry| self.operating_world(entry))
    }

    pub fn prepare_mutation_operating_world(
        &self,
    ) -> Result<
        crate::domain_installation::WorthQueryInstalledOperatingWorld<
            '_,
            crate::basis_lifecycle::MutationPreparationLaneWitness,
        >,
        crate::domain_installation::WorthQueryOperatingWorldEntryDenial,
    > {
        if let Some(installed) = &self.runtime.installed_product {
            let product = installed
                .world
                .admit_product_branch(installed.world.default_branch())
                .map_err(product_admission_denial)?;
            let entry = crate::domain_installation::WorthQueryOperatingWorldEntry::prepare_current_mutation()?;
            return Ok(self.product_operating_world(entry, product));
        }
        crate::domain_installation::WorthQueryOperatingWorldEntry::prepare_current_mutation()
            .map(|entry| self.operating_world(entry))
    }

    pub fn prepare_branch_mutation_operating_world(
        &self,
        branch_identity: crate::domain_installation::WorthQueryBranchHeadIdentity,
    ) -> Result<
        crate::domain_installation::WorthQueryInstalledOperatingWorld<
            '_,
            crate::basis_lifecycle::MutationPreparationLaneWitness,
        >,
        crate::domain_installation::WorthQueryOperatingWorldEntryDenial,
    > {
        self.require_component_operating_world()?;
        crate::domain_installation::WorthQueryOperatingWorldEntry::prepare_branch_mutation(
            &branch_identity,
        )
        .map(|entry| self.operating_world(entry))
    }

    pub fn observe_product_operating_world(
        &self,
        identity: &worth_query_execution::facade::runtime::ProductBranchIdentity,
    ) -> Result<
        crate::domain_installation::WorthQueryInstalledOperatingWorld<
            '_,
            crate::basis_lifecycle::ObservationLaneWitness,
        >,
        crate::domain_installation::WorthQueryOperatingWorldEntryDenial,
    > {
        let installed = self.runtime.installed_product.as_ref().ok_or_else(|| crate::domain_installation::WorthQueryOperatingWorldEntryDenial::product(crate::domain_installation::WorthQueryOperatingWorldProductDenial::RuntimeUnavailable))?;
        let product = installed
            .world
            .admit_product_branch(identity)
            .map_err(product_admission_denial)?;
        let entry = if identity == installed.world.default_branch() {
            crate::domain_installation::WorthQueryOperatingWorldEntry::observe_current()?
        } else {
            crate::domain_installation::WorthQueryOperatingWorldEntry::observe_product_component(
                &product,
            )?
        };
        Ok(self.product_operating_world(entry, product))
    }

    pub fn create_product_branch(
        &self,
        source: &worth_query_execution::facade::primary_graph::WorthQueryProductBranchLease,
        intent: worth_query_execution::facade::runtime::ProductBranchCreationIntent,
        cancellation: &worth_query_execution::facade::runtime::RuntimeWorldCancellationToken,
    ) -> Result<
        worth_query_execution::facade::runtime::RuntimeWorldBranchCreationOutcome,
        super::WorthQueryProductBranchCreationDenial,
    > {
        self.runtime
            .create_product_branch(source, intent, cancellation)
    }

    fn product_operating_world<L: crate::basis_lifecycle::BasisOperationLane>(
        &self,
        entry: crate::domain_installation::WorthQueryOperatingWorldEntry<L>,
        product: worth_query_execution::facade::primary_graph::WorthQueryProductBranchLease,
    ) -> crate::domain_installation::WorthQueryInstalledOperatingWorld<'_, L> {
        crate::domain_installation::WorthQueryInstalledOperatingWorld::new(
            &self.runtime,
            crate::domain_installation::WorthQueryOperatingWorldBasis::Product {
                capability: entry.into_capability(),
                product: std::sync::Arc::new(product),
            },
        )
    }

    fn require_component_operating_world(
        &self,
    ) -> Result<(), crate::domain_installation::WorthQueryOperatingWorldEntryDenial> {
        if self.runtime.installed_product.is_some() {
            return Err(crate::domain_installation::WorthQueryOperatingWorldEntryDenial::product(crate::domain_installation::WorthQueryOperatingWorldProductDenial::SelectionRequired));
        }
        Ok(())
    }

    pub fn graph_participation<G: 'static>(
        &self,
        marker: G,
    ) -> Result<
        crate::domain_installation::WorthQueryInstalledGraphParticipation<G>,
        crate::domain_installation::WorthQueryGraphParticipationLookupDenial,
    > {
        self.runtime.graph_participation(marker)
    }

    pub fn domain<D: 'static>(
        &self,
        marker: D,
    ) -> Result<
        crate::domain_installation::WorthQueryInstalledDomainHandle<D>,
        crate::domain_installation::WorthQueryDomainHandleDenial,
    > {
        self.runtime.domain(marker)
    }

    pub fn domain_installation_receipt<D: 'static>(
        &self,
        marker: D,
    ) -> Option<&crate::domain_installation::WorthQueryDomainInstallationReceipt> {
        self.runtime.domain_installation_receipt(marker)
    }

    pub fn verify_domain_execution_index_rebuild(
        &self,
    ) -> crate::domain_installation::WorthQueryDomainExecutionIndexRebuildReport {
        self.runtime.verify_domain_execution_index_rebuild()
    }

    pub fn rebuild_conditional_execution_index(
        &mut self,
    ) -> crate::domain_installation::WorthQueryConditionalExecutionIndexRebuildReport {
        self.runtime.rebuild_conditional_execution_index()
    }

    /// Rebind a prior runtime-installed domain handle into this workspace.
    ///
    /// The workspace remains the owning runtime boundary; downstream
    /// consumers never reconstruct installation generation or runtime
    /// affinity from receipt fields.
    pub fn rebind_domain<D: 'static>(
        &self,
        request: crate::domain_installation::WorthQueryDomainRebindRequest<D>,
    ) -> Result<
        crate::domain_installation::WorthQueryReboundDomainHandle<D>,
        crate::domain_installation::WorthQueryDomainRebindDenial,
    > {
        self.runtime.rebind_domain(request)
    }

    pub(crate) fn validate_installed_domain_witness<D: 'static>(
        &self,
        witness: &crate::domain_installation::WorthQueryInstalledDomainAuthorityWitness,
    ) -> Result<(), crate::domain_installation::WorthQueryDomainHandleDenial> {
        self.runtime.validate_installed_domain_witness::<D>(witness)
    }

    pub(crate) fn replace_domain_installation_with_successor_generation(
        &mut self,
    ) -> Result<(), crate::runtime::WorthQueryRuntimeError> {
        self.runtime
            .replace_domain_installation_with_successor_generation()
    }
}

fn product_admission_denial(
    denial: worth_query_execution::facade::primary_graph::WorthQueryProductBranchAdmissionDenial,
) -> crate::domain_installation::WorthQueryOperatingWorldEntryDenial {
    crate::domain_installation::WorthQueryOperatingWorldEntryDenial::product(
        crate::domain_installation::WorthQueryOperatingWorldProductDenial::Admission(denial),
    )
}
