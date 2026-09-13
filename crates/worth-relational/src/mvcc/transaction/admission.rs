#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelationalBranchTransactionAdmissionDenial {
    OwnerUnavailable,
    ForeignRuntime {
        expected_runtime_instance_id: u64,
        actual_runtime_instance_id: u64,
    },
    BasisIdentityMismatch,
    UnknownBranch,
    Archived,
    Deleting,
    StaleBasis,
    RetentionCapacityExhausted,
    RetentionOwnerUnavailable,
    RetentionIdentityExhausted,
    RetentionInvariantViolation,
    Cancelled,
    TimedOut,
}

impl crate::runtime::RelationalRuntime {
    pub fn begin_branch_transaction(
        &self,
        basis: &crate::branch::AdmittedRelationalBranchBasis,
        intent: super::RelationalTransactionIntent,
    ) -> Result<super::BranchBoundRelationalTransaction, RelationalBranchTransactionAdmissionDenial>
    {
        self.begin_branch_transaction_with_control(
            basis,
            intent,
            crate::mvcc::RelationalOperationControl::uninterrupted(),
        )
    }

    pub fn begin_branch_transaction_with_control(
        &self,
        basis: &crate::branch::AdmittedRelationalBranchBasis,
        intent: super::RelationalTransactionIntent,
        control: crate::mvcc::RelationalOperationControl,
    ) -> Result<super::BranchBoundRelationalTransaction, RelationalBranchTransactionAdmissionDenial>
    {
        if basis.identity().runtime_instance_id() != self.runtime_instance_id() {
            return Err(RelationalBranchTransactionAdmissionDenial::ForeignRuntime {
                expected_runtime_instance_id: self.runtime_instance_id(),
                actual_runtime_instance_id: basis.identity().runtime_instance_id(),
            });
        }
        if basis.descriptor().runtime_instance_id() != self.runtime_instance_id()
            || basis.descriptor().branch_id() != basis.identity().branch_id()
            || basis.descriptor().root_identity() != basis.inner.root.id()
        {
            return Err(RelationalBranchTransactionAdmissionDenial::BasisIdentityMismatch);
        }
        let cell = self
            .history
            .branch_cell(basis.identity().branch_id())
            .filter(|cell| cell.identity() == basis.identity())
            .ok_or(RelationalBranchTransactionAdmissionDenial::UnknownBranch)?;
        let publication_cell = cell.publication_cell();
        let _coordination = publication_cell.coordination().enter();
        match publication_cell.enter_state().lifecycle_posture() {
            crate::branch::RelationalBranchLifecyclePosture::Live => {}
            crate::branch::RelationalBranchLifecyclePosture::Archived => {
                return Err(RelationalBranchTransactionAdmissionDenial::Archived);
            }
            crate::branch::RelationalBranchLifecyclePosture::Deleting => {
                return Err(RelationalBranchTransactionAdmissionDenial::Deleting);
            }
        }
        if !basis.is_current() {
            return Err(RelationalBranchTransactionAdmissionDenial::StaleBasis);
        }
        match control.observe(crate::mvcc::RelationalInterruptionBoundary::TransactionAdmission) {
            Some(event)
                if event.interruption()
                    == crate::mvcc::RelationalOperationInterruption::Cancelled =>
            {
                basis.inner.retention_binding.record_interruption(event);
                return Err(RelationalBranchTransactionAdmissionDenial::Cancelled);
            }
            Some(event) => {
                basis.inner.retention_binding.record_interruption(event);
                return Err(RelationalBranchTransactionAdmissionDenial::TimedOut);
            }
            None => {}
        }
        let retention =
            crate::history::retention::RelationalTransactionRetentionObligation::acquire(
                &basis.inner.retention_binding,
                basis.identity().clone(),
                std::sync::Arc::clone(&basis.inner.root),
            )
            .map_err(|denial| {
                match denial {
            crate::history::retention::RelationalRetentionAcquisitionDenial::CapacityExhausted => {
                RelationalBranchTransactionAdmissionDenial::RetentionCapacityExhausted
            }
            crate::history::retention::RelationalRetentionAcquisitionDenial::OwnerUnavailable => {
                RelationalBranchTransactionAdmissionDenial::RetentionOwnerUnavailable
            }
            crate::history::retention::RelationalRetentionAcquisitionDenial::IdentityExhausted => {
                RelationalBranchTransactionAdmissionDenial::RetentionIdentityExhausted
            }
            crate::history::retention::RelationalRetentionAcquisitionDenial::RootSetTooLarge => {
                RelationalBranchTransactionAdmissionDenial::RetentionInvariantViolation
            }
        }
            })?;

        match control.observe(crate::mvcc::RelationalInterruptionBoundary::TransactionAdmission) {
            Some(event)
                if event.interruption()
                    == crate::mvcc::RelationalOperationInterruption::Cancelled =>
            {
                basis.inner.retention_binding.record_interruption(event);
                drop(retention);
                return Err(RelationalBranchTransactionAdmissionDenial::Cancelled);
            }
            Some(event) => {
                basis.inner.retention_binding.record_interruption(event);
                drop(retention);
                return Err(RelationalBranchTransactionAdmissionDenial::TimedOut);
            }
            None => {}
        }

        let footprint = super::RelationalTransactionFootprint::for_basis(basis);
        Ok(super::BranchBoundRelationalTransaction {
            basis: basis.clone(),
            mutation_authority: crate::branch::issue_relational_branch_mutation_authority(),
            transaction_id: self.services.next_transaction_id(),
            intent,
            merge_parent_bases: Vec::new(),
            schema_authority_input: None,
            schema_authority: basis.inner.root.retained_schema_authority(),
            overlay: super::DetachedRelationalTransactionOverlay::default(),
            overlay_bytes: 0,
            maximum_overlay_bytes: self.config.publication.policy.max_transaction_overlay_bytes,
            maximum_footprint_loci: self
                .config
                .publication
                .policy
                .max_transaction_footprint_loci,
            maximum_savepoints: self.config.publication.policy.max_transaction_savepoints,
            savepoint_footprint_loci: 0,
            footprint,
            savepoints: Vec::new(),
            next_savepoint_ordinal: 1,
            last_merged_plan: None,
            client_key_symbol_policy: self.config.identity.client_key_symbol_policy,
            retention: std::sync::Arc::new(retention),
            control,
        })
    }

    pub(crate) fn begin_branch_transaction_with_owner_inputs(
        &self,
        owner_inputs: crate::mvcc::RelationalTransactionValidationInput,
    ) -> Result<super::BranchBoundRelationalTransaction, RelationalBranchTransactionAdmissionDenial>
    {
        let mut transaction =
            self.begin_branch_transaction(owner_inputs.basis(), owner_inputs.intent().clone())?;
        owner_inputs.apply_owner_inputs_to(&mut transaction);
        Ok(transaction)
    }
}
