use worth_foundational::facade::{
    export_portable_record_aspect_state, readmit_portable_record_aspect_state, AspectContract,
    AspectContractRevision, AspectIdentity, AspectKey, AuthoritativeRecordAspectState,
    PortableAspectContract, PortableAspectContractLookup, PortableRecordAspectState,
};
use worth_proof::TransitionOutcome;

use crate::durability::data::{DurabilityError, RecoveryFailureClass};
pub(super) fn export_state(
    state: Option<AuthoritativeRecordAspectState>,
    contracts: Option<&CheckpointAspectContractCatalog>,
) -> Result<Option<PortableRecordAspectState>, DurabilityError> {
    state
        .map(|state| {
            let contracts = contracts.ok_or_else(|| {
                DurabilityError::new(
                    RecoveryFailureClass::SchemaMismatch,
                    "checkpoint aspect state belongs to a slot without a kind plan",
                )
            })?;
            export_portable_record_aspect_state(&state, contracts).map_err(|denial| {
                DurabilityError::new(
                    RecoveryFailureClass::SchemaMismatch,
                    format!("checkpoint aspect export denied: {denial:?}"),
                )
            })
        })
        .transpose()
}

pub(super) fn readmit_state(
    state: Option<PortableRecordAspectState>,
    contracts: &CheckpointAspectContractCatalog,
) -> Result<Option<AuthoritativeRecordAspectState>, DurabilityError> {
    state
        .map(
            |state| match readmit_portable_record_aspect_state(state, contracts) {
                TransitionOutcome::Success(artifact) => Ok(artifact.into_parts().into_parts().0),
                TransitionOutcome::Denied(denial) => Err(DurabilityError::new(
                    RecoveryFailureClass::CorruptCheckpoint,
                    format!("checkpoint aspect readmission denied: {denial:?}"),
                )),
            },
        )
        .transpose()
}

#[derive(PartialEq, Eq)]
pub(crate) struct CheckpointAspectContractCatalog {
    contracts: std::collections::BTreeMap<
        (AspectKey, AspectIdentity, AspectContractRevision),
        AspectContract,
    >,
}

impl CheckpointAspectContractCatalog {
    pub(crate) fn readmit(candidates: &[PortableAspectContract]) -> Result<Self, DurabilityError> {
        let mut contracts = std::collections::BTreeMap::new();
        for candidate in candidates {
            let contract = candidate.readmit().map_err(|denial| {
                DurabilityError::new(
                    RecoveryFailureClass::CorruptCheckpoint,
                    format!("checkpoint aspect contract readmission denied: {denial:?}"),
                )
            })?;
            let basis = contract_basis(&contract);
            match contracts.get(&basis) {
                Some(existing) if existing == &contract => continue,
                Some(_) => {
                    return Err(DurabilityError::new(
                        RecoveryFailureClass::CorruptCheckpoint,
                        format!(
                            "checkpoint carries conflicting contracts for aspect `{}`",
                            contract.key().as_str()
                        ),
                    ));
                }
                None => {
                    contracts.insert(basis, contract);
                }
            }
        }
        Ok(Self { contracts })
    }

    pub(crate) fn from_plans(
        plans: &crate::schema::data::AspectContractPlanCatalog,
    ) -> Result<Self, DurabilityError> {
        let contracts = plans
            .entity_plans
            .values()
            .chain(plans.relation_plans.values())
            .flat_map(|plan| plan.executable_bindings.iter())
            .map(|binding| binding.contract.clone())
            .collect::<Vec<_>>();
        Self::from_contracts(&contracts)
    }

    pub(crate) fn from_contracts(contracts: &[AspectContract]) -> Result<Self, DurabilityError> {
        let candidates = contracts
            .iter()
            .map(PortableAspectContract::from_contract)
            .collect::<Vec<_>>();
        Self::readmit(&candidates)
    }
}

impl PortableAspectContractLookup for CheckpointAspectContractCatalog {
    fn contract_for(&self, key: &AspectKey) -> Option<AspectContract> {
        self.contracts
            .iter()
            .rev()
            .find(|((candidate, _, _), _)| candidate == key)
            .map(|(_, contract)| contract.clone())
    }

    fn exact_contract_for(
        &self,
        key: &AspectKey,
        identity: AspectIdentity,
        revision: AspectContractRevision,
    ) -> Option<AspectContract> {
        self.contracts
            .get(&(key.clone(), identity, revision))
            .cloned()
    }
}

fn contract_basis(
    contract: &AspectContract,
) -> (AspectKey, AspectIdentity, AspectContractRevision) {
    (
        contract.key().clone(),
        contract.identity(),
        contract.revision(),
    )
}
