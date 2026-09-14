use super::{
    PortableAspectContractBasis, PortableAspectContractLookup, PortableAspectExportDenial,
    PortableAspectReadmissionDenial,
};
use crate::aspects::{AspectContract, AspectContractRevision, AspectIdentity, AspectKey};

pub(super) fn contract_for_readmission(
    basis: &PortableAspectContractBasis,
    contracts: &impl PortableAspectContractLookup,
) -> Result<AspectContract, PortableAspectReadmissionDenial> {
    let contract = contracts
        .exact_contract_for(basis.key(), basis.identity(), basis.revision())
        .or_else(|| contracts.contract_for(basis.key()))
        .ok_or_else(|| PortableAspectReadmissionDenial::MissingContract(basis.key().clone()))?;

    if contract.identity() != basis.identity() {
        return Err(PortableAspectReadmissionDenial::ContractIdentityMismatch {
            key: basis.key().clone(),
            expected: contract.identity(),
            found: basis.identity(),
        });
    }
    if contract.revision() != basis.revision() {
        return Err(PortableAspectReadmissionDenial::ContractRevisionMismatch {
            key: basis.key().clone(),
            expected: contract.revision(),
            found: basis.revision(),
        });
    }

    Ok(contract)
}

pub(super) fn contract_for_export(
    key: &AspectKey,
    identity: AspectIdentity,
    revision: AspectContractRevision,
    contracts: &impl PortableAspectContractLookup,
) -> Result<AspectContract, PortableAspectExportDenial> {
    let contract = contracts
        .exact_contract_for(key, identity, revision)
        .or_else(|| contracts.contract_for(key))
        .ok_or_else(|| PortableAspectExportDenial::MissingContract(key.clone()))?;
    if contract.identity() != identity {
        return Err(PortableAspectExportDenial::ContractIdentityDrift {
            key: key.clone(),
            expected: identity,
            found: contract.identity(),
        });
    }
    if contract.revision() != revision {
        return Err(PortableAspectExportDenial::ContractRevisionDrift {
            key: key.clone(),
            expected: revision,
            found: contract.revision(),
        });
    }
    Ok(contract)
}

pub(super) fn exact_contract_for_export(
    expected: &AspectContract,
    contracts: &impl PortableAspectContractLookup,
) -> Result<AspectContract, PortableAspectExportDenial> {
    contract_for_export(
        expected.key(),
        expected.identity(),
        expected.revision(),
        contracts,
    )
}
