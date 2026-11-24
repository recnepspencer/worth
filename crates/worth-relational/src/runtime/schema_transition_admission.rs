use crate::runtime::RelationalRuntime;
use crate::schema::data::{
    ProposedSchemaTransition, RelationalSchemaRegistry, SchemaId, SchemaReconciliationPolicy,
    SchemaVersionId,
};
use worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasisFieldsPort as _;

use crate::durability::transition_admission_denial::{
    RelationalSchemaTransitionAdmissionDenial, RelationalSchemaTransitionAdmissionDenialKind,
};

impl RelationalRuntime {
    pub fn begin_branch_schema_transition(
        &self,
        basis: &crate::branch::AdmittedRelationalBranchBasis,
        transition: ProposedSchemaTransition,
        reconciliation_policy: Option<SchemaReconciliationPolicy>,
        target_registry: RelationalSchemaRegistry,
    ) -> Result<
        crate::mvcc::BranchBoundRelationalTransaction,
        RelationalSchemaTransitionAdmissionDenial,
    > {
        self.begin_branch_schema_transition_with_control(
            basis,
            transition,
            reconciliation_policy,
            target_registry,
            crate::mvcc::RelationalOperationControl::uninterrupted(),
        )
    }

    pub fn begin_branch_schema_transition_with_control(
        &self,
        basis: &crate::branch::AdmittedRelationalBranchBasis,
        transition: ProposedSchemaTransition,
        reconciliation_policy: Option<SchemaReconciliationPolicy>,
        target_registry: RelationalSchemaRegistry,
        control: crate::mvcc::RelationalOperationControl,
    ) -> Result<
        crate::mvcc::BranchBoundRelationalTransaction,
        RelationalSchemaTransitionAdmissionDenial,
    > {
        let source_authority = basis.owner_inner().root.schema_authority();
        let source = registry_basis(source_authority.registry()).map_err(|detail| {
            RelationalSchemaTransitionAdmissionDenial::new(
                RelationalSchemaTransitionAdmissionDenialKind::SourceBasisMismatch,
                detail,
            )
        })?;
        let declared_source = (
            transition.source_schema_id.clone(),
            transition.source_schema_version_id,
        );
        if source != declared_source {
            return Err(RelationalSchemaTransitionAdmissionDenial::new(
                RelationalSchemaTransitionAdmissionDenialKind::SourceBasisMismatch,
                format!("branch source {source:?} does not match {declared_source:?}"),
            ));
        }

        let target = registry_basis(&target_registry).map_err(|detail| {
            RelationalSchemaTransitionAdmissionDenial::new(
                RelationalSchemaTransitionAdmissionDenialKind::InvalidTargetRegistry,
                detail,
            )
        })?;
        let declared_target = (
            transition.target_schema_id.clone(),
            transition.target_schema_version_id,
        );
        if target != declared_target {
            return Err(RelationalSchemaTransitionAdmissionDenial::new(
                RelationalSchemaTransitionAdmissionDenialKind::TargetBasisMismatch,
                format!("target registry {target:?} does not match {declared_target:?}"),
            ));
        }
        if !source_authority.has_same_executable_meaning_except_basis(&target_registry) {
            return Err(RelationalSchemaTransitionAdmissionDenial::new(
                RelationalSchemaTransitionAdmissionDenialKind::TargetContractMismatch,
                "target registry does not preserve the source root's complete executable kind contracts",
            ));
        }

        let intent = crate::mvcc::RelationalTransactionIntent::ordinary()
            .with_schema_transition(transition, reconciliation_policy);
        let mut transaction = self
            .begin_branch_transaction_with_control(basis, intent, control)
            .map_err(|denial| {
                RelationalSchemaTransitionAdmissionDenial::new(
                    RelationalSchemaTransitionAdmissionDenialKind::TransactionAdmission,
                    format!("{denial:?}"),
                )
            })?;
        transaction.schema_authority_input = Some(
            crate::schema::SchemaContinuityAuthorityInput::from_registry(
                target_registry,
                source_authority.descriptor_semantics_version(),
                self.config
                    .schema
                    .descriptor_canonical_basis_policy
                    .current_write_version(),
            ),
        );
        Ok(transaction)
    }
}

pub(super) fn registry_basis(
    registry: &RelationalSchemaRegistry,
) -> Result<(SchemaId, SchemaVersionId), String> {
    registry
        .authoritative_schema_basis()
        .map_err(|error| error.detail)?
        .ok_or_else(|| "schema registry has no authoritative basis".to_owned())
}
