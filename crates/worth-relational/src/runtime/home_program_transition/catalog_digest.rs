//! The complete compiled target is committed before any migration can run.
use super::RelationalHomeProgramContractDenial as Denial;
use crate::validation::candidate_source::{
    ManagedIndexSourceProgram, SourceProgramCommitmentDenial,
};
use crate::validation::data::CustomInvariantRegistration;
use sha2::{Digest, Sha256};
use worth_execution::ExecutionRequest;
use worth_foundational::facade::HomeProgramTransitionCatalogDigest;
#[cfg(test)]
mod tests;

pub(super) fn digest(
    schema: [u8; 32],
    invariants: &[CustomInvariantRegistration],
    sources: &[ManagedIndexSourceProgram],
    request: &ExecutionRequest<'_, '_>,
) -> Result<HomeProgramTransitionCatalogDigest, Denial> {
    digest_programs(schema, invariants, sources.iter(), request)
}

pub(super) fn digest_frozen(
    schema: [u8; 32],
    frozen: &crate::validation::FrozenCustomInvariantRegistry,
    request: &ExecutionRequest<'_, '_>,
) -> Result<HomeProgramTransitionCatalogDigest, Denial> {
    digest_programs(
        schema,
        frozen.registrations(),
        frozen
            .all_source_groups()
            .iter()
            .flat_map(crate::validation::candidate_source::SourcePreparationGroup::public_programs),
        request,
    )
}

fn digest_programs<'source>(
    schema: [u8; 32],
    invariants: &[CustomInvariantRegistration],
    sources: impl Iterator<Item = &'source ManagedIndexSourceProgram> + Clone,
    request: &ExecutionRequest<'_, '_>,
) -> Result<HomeProgramTransitionCatalogDigest, Denial> {
    let mut source_count = 0usize;
    for _ in sources.clone() {
        request.consult().map_err(Denial::Request)?;
        source_count = source_count
            .checked_add(1)
            .ok_or(Denial::CapacityOverflow)?;
    }
    let count = invariants
        .len()
        .checked_add(source_count)
        .ok_or(Denial::CapacityOverflow)?;
    let bytes = count
        .checked_mul(size_of::<[u8; 32]>())
        .ok_or(Denial::CapacityOverflow)?;
    let _memory = request
        .reserve_retained_memory(bytes as u64)
        .map_err(Denial::Request)?;
    let mut requirements: Vec<[u8; 32]> = Vec::with_capacity(count);
    for invariant in invariants {
        request.consult().map_err(Denial::Request)?;
        let mut hash = Sha256::new();
        hash.update(b"invariant-source-requirements\0");
        let identity = &invariant.descriptor().identity;
        hash.update((identity.rule_id.as_str().len() as u64).to_le_bytes());
        hash.update(identity.rule_id.as_str().as_bytes());
        hash.update(identity.semantic_version.major.to_le_bytes());
        hash.update(identity.semantic_version.minor.to_le_bytes());
        hash.update([invariant.execution_point() as u8]);
        hash.update([u8::from(
            invariant
                .access_contract()
                .include_relation_endpoint_entity_touches,
        )]);
        hash.update((invariant.managed_sources.len() as u64).to_le_bytes());
        for source in invariant.managed_sources.iter() {
            hash.update(program_digest(source, request)?);
        }
        requirements.push(hash.finalize().into());
    }
    for source in sources {
        let mut hash = Sha256::new();
        hash.update(b"public-source-requirement\0");
        hash.update(program_digest(source, request)?);
        requirements.push(hash.finalize().into());
    }
    request.consult().map_err(Denial::Request)?;
    requirements.sort_unstable();
    request.consult().map_err(Denial::Request)?;
    let mut hash = Sha256::new();
    hash.update(b"worth.native.home.execution-catalog.v1\0");
    hash.update(schema);
    hash.update(
        crate::runtime::custom_invariant_inventory_digest_for_compilation(
            invariants,
            crate::schema::SchemaCompilationControl::Request(*request),
        )
        .map_err(Denial::Execution)?,
    );
    hash.update((requirements.len() as u64).to_le_bytes());
    for requirement in requirements {
        hash.update(requirement);
    }
    Ok(HomeProgramTransitionCatalogDigest::from_recorded(
        hash.finalize().into(),
    ))
}

fn program_digest(
    source: &ManagedIndexSourceProgram,
    request: &ExecutionRequest<'_, '_>,
) -> Result<[u8; 32], Denial> {
    source
        .catalog_commitment(request)
        .map_err(|cause| match cause {
            SourceProgramCommitmentDenial::Request(cause) => Denial::Request(cause),
            SourceProgramCommitmentDenial::Encoding => Denial::CatalogEncoding,
        })
}
