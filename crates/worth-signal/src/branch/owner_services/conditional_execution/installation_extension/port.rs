use super::*;

impl<D, I, T> SignalConditionalExecutionPort<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    #[cfg(test)]
    pub(super) fn set_installation_work_budget_for_test(&self, maximum_visits: usize) {
        let owner = SignalOwner::upgrade(&self.owner).expect("test Signal owner remains available");
        let admission = owner.admit().expect("test Signal owner admits work");
        let cell = owner
            .lookup_cell(&admission, self.basis.owner_branch_id())
            .expect("test Signal branch remains registered");
        cell.set_conditional_installation_work_budget_for_test(maximum_visits);
    }

    pub fn prepare_installation_extension(
        &self,
        node: NodeId,
        expected_contract_generation: u64,
        definition: SignalConditionalContractDefinition,
    ) -> Result<
        SignalPreparedConditionalInstallationExtension,
        SignalConditionalInstallationExtensionDenial,
    > {
        self.prepare_extension(
            SignalConditionalInstallationTarget::Existing {
                node,
                expected_contract_generation,
            },
            definition,
        )
    }

    pub fn prepare_owned_installation_extension(
        &self,
        definition: SignalConditionalContractDefinition,
    ) -> Result<
        SignalPreparedConditionalInstallationExtension,
        SignalConditionalInstallationExtensionDenial,
    > {
        self.prepare_extension(SignalConditionalInstallationTarget::Allocate, definition)
    }

    fn prepare_extension(
        &self,
        target: SignalConditionalInstallationTarget,
        definition: SignalConditionalContractDefinition,
    ) -> Result<
        SignalPreparedConditionalInstallationExtension,
        SignalConditionalInstallationExtensionDenial,
    > {
        use SignalConditionalInstallationExtensionDenial as Denial;
        let owner = SignalOwner::upgrade(&self.owner).map_err(Denial::OwnerUnavailable)?;
        if !owner.basis_has_owner_affinity(&self.basis) {
            return Err(Denial::ForeignBasis);
        }
        let admission = owner
            .admit()
            .map_err(map_observation_admission_denial)
            .map_err(Denial::OwnerAdmission)?;
        let branch = self.basis.owner_branch_id();
        let cell = owner
            .lookup_cell(&admission, branch)
            .map_err(|denial| Denial::OwnerAdmission(map_basis_registry_denial(denial, branch)))?;
        if cell.incarnation() != self.incarnation {
            return Err(Denial::StaleCell);
        }
        let (prepared, custody) = cell.prepare_conditional_installation(
            &admission,
            &self.basis,
            &self.definition,
            &self.claimant,
            target,
            definition,
            &owner.conditional_retention,
        )?;
        let ordinal = self
            .next_installation_ordinal
            .fetch_update(
                std::sync::atomic::Ordering::Relaxed,
                std::sync::atomic::Ordering::Relaxed,
                |current| current.checked_add(1),
            )
            .map_err(|_| Denial::PublicationOrdinalExhausted)?;
        let scope = SignalConditionalDefinitionPublicationScope::issue(
            Arc::clone(&self.authority),
            ordinal,
        );
        let operation = SignalConditionalDefinitionPublicationOperation {
            predecessor: self.basis.clone(),
            incarnation: self.incarnation,
            scope: scope.clone(),
        };
        let request = SignalConditionalInstallationExtensionRequest {
            prepared,
            service_authority: Arc::clone(&self.authority),
            definition_binding: self.definition.clone(),
            publication_scope: scope,
            predecessor: self.basis.clone(),
            incarnation: self.incarnation,
            custody,
        };
        Ok(SignalPreparedConditionalInstallationExtension { operation, request })
    }

    pub fn apply_installation_extension<E, Ctx>(
        &self,
        transaction: &mut SignalTransaction<'_, D, I, E, Ctx, T>,
        request: SignalConditionalInstallationExtensionRequest,
    ) -> Result<
        SignalConditionalInstallationExtensionCompletion<D, I, T>,
        SignalConditionalInstallationExtensionDenial,
    > {
        use SignalConditionalInstallationExtensionDenial as Denial;
        if !Arc::ptr_eq(&self.authority, &request.service_authority) {
            return Err(Denial::ForeignService);
        }
        if !self.definition.matches(&request.definition_binding) {
            return Err(Denial::DefinitionMismatch);
        }
        if !transaction.admits_conditional_operation_scope(self)
            || !transaction.admits_conditional_definition_publication(&request.publication_scope)
        {
            return Err(Denial::TransactionScopeMismatch);
        }
        let (contract, predecessor_generation, predecessor_occurrence) = match request.prepared {
            SignalPreparedInstallationTarget::Existing(prepared) => {
                let predecessor_generation = prepared.predecessor_generation();
                let predecessor_occurrence = prepared.predecessor_occurrence();
                let contract = transaction
                    .apply_prepared_conditional_contract_within_owner(prepared)
                    .map_err(Denial::SignalMutation)?;
                (contract, predecessor_generation, predecessor_occurrence)
            }
            SignalPreparedInstallationTarget::Allocate(definition) => {
                let node = transaction.create_conditional_installation_node();
                let contract = transaction
                    .install_conditional_contract_within_owner(&self.claimant, node, definition)
                    .map_err(Denial::SignalMutation)?;
                (contract, 0, 0)
            }
        };
        let successor_service = self.prepare_successor_service(transaction)?;
        Ok(SignalConditionalInstallationExtensionCompletion {
            contract,
            predecessor_generation,
            predecessor_occurrence,
            predecessor: request.predecessor,
            incarnation: request.incarnation,
            service_authority: request.service_authority,
            publication_scope: request.publication_scope,
            custody: request.custody,
            successor_service,
        })
    }

    fn prepare_successor_service<E, Ctx>(
        &self,
        transaction: &mut SignalTransaction<'_, D, I, E, Ctx, T>,
    ) -> Result<
        SignalPreparedConditionalExecutionService<D, I, T>,
        SignalConditionalInstallationExtensionDenial,
    > {
        use super::super::SignalConditionalBasisCaptureDenial as CaptureDenial;
        use SignalConditionalInstallationExtensionDenial as Denial;
        let owner = SignalOwner::upgrade(&self.owner).map_err(Denial::OwnerUnavailable)?;
        let maximum_visits = transaction
            .conditional_execution_graph()
            .installed_runtime_policy()
            .conditional_evaluation_budget()
            .maximum_attempt_visits;
        let retained = super::super::SignalRetainedExecutionBasis::capture(
            transaction.conditional_execution_graph_mut(),
            &owner.conditional_retention,
            &mut RetainedStoragePreparation::new(maximum_visits),
        )
        .map_err(|denial| match denial {
            CaptureDenial::CapacityExhausted => Denial::CapacityExhausted,
            CaptureDenial::WorkExhausted { maximum_visits } => {
                Denial::WorkExhausted { maximum_visits }
            }
            CaptureDenial::OwnerUnavailable | CaptureDenial::Unavailable => {
                Denial::RetentionUnavailable
            }
        })?;
        Ok(SignalPreparedConditionalExecutionService {
            owner: self.owner.clone(),
            definition: self.definition.clone(),
            incarnation: self.incarnation,
            authority: Arc::clone(&self.authority),
            source_authority: self.source_authority.clone(),
            claimant: self.claimant.clone(),
            issuance_basis_custody: Arc::new(retained),
        })
    }
}
