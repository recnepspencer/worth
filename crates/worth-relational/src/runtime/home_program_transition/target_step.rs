//! Target-only whole-candidate execution payload under an admitted Home step.
use super::*;
use crate::schema::data::{ProposedSchemaTransition, SchemaReconciliationPolicy};
use worth_relational_durability::internal_port::durability::transition_admission_denial::RelationalSchemaTransitionAdmissionDenialOwnerPort as _;

#[derive(Debug)]
pub(crate) struct AdmittedRelationalHomeProgramTarget {
    pub(crate) step: worth_foundational::facade::HomeProgramTransitionStepAssociation,
    pub(crate) meaning: Arc<RelationalRootExecutionContracts>,
    pub(crate) receiver_epoch: crate::runtime::RelationalRuntimeConfigurationSnapshot,
    pub(crate) index_definitions: Option<Arc<crate::runtime::PreparedRelationalIndexDefinitions>>,
    _memory: Option<ExecutionMemoryReservation>,
}

impl AdmittedRelationalHomeProgramTarget {
    pub(crate) fn for_preflight(target: &PreparedRelationalHomeProgramContracts) -> Arc<Self> {
        let home = worth_foundational::facade::HomeIdentity::from_observed_bytes([1u8; 16])
            .expect("non-zero sentinel");
        let transition = worth_foundational::facade::HomeProgramTransitionIdentity::from_recorded(
            home,
            std::num::NonZeroU64::MIN,
        );
        Arc::new(Self {
            step: worth_foundational::facade::HomeProgramTransitionStepAssociation::from_recorded(
                transition, 0,
            ),
            meaning: target.meaning.clone(),
            receiver_epoch: target.receiver_epoch.clone(),
            index_definitions: target.index_definitions.clone(),
            _memory: None,
        })
    }
}

impl RelationalRuntime {
    pub(crate) fn prepare_home_target_transaction(
        &self,
        request: &ExecutionRequest<'_, '_>,
        basis: &AdmittedRelationalBranchBasis,
        target: &PreparedRelationalHomeProgramContracts,
        transition: Option<(ProposedSchemaTransition, Option<SchemaReconciliationPolicy>)>,
        step: Arc<AdmittedRelationalHomeProgramStep>,
    ) -> Result<crate::mvcc::BranchBoundRelationalTransaction, RelationalHomeProgramContractDenial>
    {
        use RelationalHomeProgramContractDenial as Denial;
        let receiver_epoch = target.receiver_epoch.clone();
        let index_definitions = target.index_definitions.clone();
        let target = target.meaning.clone();
        if target.runtime != self.runtime_instance_id()
            || basis.identity().runtime_instance_id() != self.runtime_instance_id()
        {
            return Err(Denial::ForeignRuntime);
        }
        let epoch = self.configuration_binding().snapshot();
        if !receiver_epoch.same_epoch(&epoch) {
            return Err(Denial::SourceContractChanged);
        }
        let mut intent = crate::mvcc::RelationalTransactionIntent::ordinary();
        if let Some((transition, policy)) = transition {
            require_bases(basis, &target, &transition)?;
            intent = intent.with_schema_transition(transition, policy);
        } else if basis
            .owner_inner()
            .root
            .schema_authority()
            .authority_digest()
            != target.schema.authority_digest()
        {
            return Err(Denial::SchemaCapture);
        }
        let layout = std::alloc::Layout::new::<[usize; 2]>()
            .extend(std::alloc::Layout::new::<AdmittedRelationalHomeProgramTarget>())
            .map_err(|_| Denial::CapacityOverflow)?
            .0
            .pad_to_align();
        let memory = request
            .reserve_retained_memory(layout.size() as u64)
            .map_err(Denial::Request)?;
        let target_step = step.step;
        let mut transaction = self
            .begin_branch_transaction_with_scope(
                basis,
                intent,
                crate::mvcc::RelationalOperationControl::uninterrupted(),
                Some(step),
                None,
            )
            .map_err(Denial::Transaction)?;
        transaction.schema_authority_input = Some(
            target.schema.continuity_input(
                epoch
                    .config
                    .schema
                    .descriptor_canonical_basis_policy
                    .current_write_version(),
            ),
        );
        transaction.home_target = Some(Arc::new(AdmittedRelationalHomeProgramTarget {
            step: target_step,
            meaning: target,
            receiver_epoch,
            index_definitions,
            _memory: memory,
        }));
        Ok(transaction)
    }
}

fn require_bases(
    source: &AdmittedRelationalBranchBasis,
    target: &RelationalRootExecutionContracts,
    transition: &ProposedSchemaTransition,
) -> Result<(), RelationalHomeProgramContractDenial> {
    use super::super::schema_transition_admission::registry_basis;
    use crate::durability::transition_admission_denial::{
        RelationalSchemaTransitionAdmissionDenial as Cause,
        RelationalSchemaTransitionAdmissionDenialKind as Kind,
    };
    let actual_source = registry_basis(source.owner_inner().root.schema_authority().registry())
        .map_err(|detail| {
            RelationalHomeProgramContractDenial::SchemaTransition(Cause::new(
                Kind::SourceBasisMismatch,
                detail,
            ))
        })?;
    let actual_target = registry_basis(target.schema.registry()).map_err(|detail| {
        RelationalHomeProgramContractDenial::SchemaTransition(Cause::new(
            Kind::TargetBasisMismatch,
            detail,
        ))
    })?;
    if actual_source
        != (
            transition.source_schema_id.clone(),
            transition.source_schema_version_id,
        )
        || actual_target
            != (
                transition.target_schema_id.clone(),
                transition.target_schema_version_id,
            )
    {
        return Err(RelationalHomeProgramContractDenial::SchemaTransition(
            Cause::new(
                Kind::TargetBasisMismatch,
                "Home schema step differs from its actual source or compiled target",
            ),
        ));
    }
    Ok(())
}
