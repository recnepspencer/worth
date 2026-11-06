//! Canonical invariant meaning uses one borrowed, admitted ordering roster.
use crate::execution::RelationalExecutionDenialCause as Denial;
use crate::schema::SchemaCompilationControl;
use crate::validation::data::{
    CustomInvariantAccessContract, CustomInvariantRegistration, CustomInvariantSemanticVersion,
    InvariantCostClass, InvariantExecutionPoint, InvariantFailureEffect,
};
use sha2::{Digest, Sha256};

type BorrowedIdentity<'a> = (
    &'a str,
    CustomInvariantSemanticVersion,
    InvariantExecutionPoint,
    &'a str,
    u32,
    InvariantCostClass,
    InvariantFailureEffect,
    Option<std::num::NonZeroU64>,
    &'a CustomInvariantAccessContract,
);

pub fn custom_invariant_inventory_digest(
    registrations: &[CustomInvariantRegistration],
) -> [u8; 32] {
    custom_invariant_inventory_digest_for_compilation(
        registrations,
        SchemaCompilationControl::Configuration,
    )
    .expect("configuration inventory digest has no request refusal")
}
pub(crate) fn custom_invariant_inventory_digest_for_compilation(
    registrations: &[CustomInvariantRegistration],
    control: SchemaCompilationControl<'_, '_>,
) -> Result<[u8; 32], Denial> {
    let bytes = registrations
        .len()
        .checked_mul(size_of::<BorrowedIdentity<'_>>())
        .ok_or(Denial::MemoryOverflow)?;
    let _memory = control.reserve(bytes)?;
    let mut identities: Vec<BorrowedIdentity<'_>> = Vec::with_capacity(registrations.len());
    for registration in registrations {
        control.checkpoint()?;
        let descriptor = registration.descriptor();
        identities.push((
            descriptor.identity.rule_id.as_str(),
            descriptor.identity.semantic_version,
            registration.execution_point(),
            &descriptor.display_name,
            registration.groups().mask(),
            registration.cost_class(),
            registration.failure_effect(),
            registration.maximum_work_units(),
            &descriptor.operational.access,
        ));
    }
    control.checkpoint()?;
    identities.sort_unstable();
    control.checkpoint()?;
    let mut digest = Sha256::new();
    for (rule, version, point, display_name, groups, cost, failure, work, access) in identities {
        control.checkpoint()?;
        digest.update((rule.len() as u64).to_le_bytes());
        mix_text(&mut digest, rule, control)?;
        digest.update(version.major.to_le_bytes());
        digest.update(version.minor.to_le_bytes());
        digest.update([point as u8]);
        digest.update((display_name.len() as u64).to_le_bytes());
        mix_text(&mut digest, display_name, control)?;
        digest.update(groups.to_le_bytes());
        digest.update([cost as u8, failure as u8]);
        match work {
            Some(work) => digest.update(work.get().to_le_bytes()),
            None => digest.update(b"work-ceiling:absent"),
        }
        for kinds in [
            &access.read_entity_kinds,
            &access.read_relation_kinds,
            &access.affected_entity_kinds,
            &access.affected_relation_kinds,
        ] {
            digest.update((kinds.len() as u64).to_le_bytes());
            for kind in kinds {
                control.checkpoint()?;
                digest.update(kind.0.to_le_bytes());
            }
        }
    }
    control.checkpoint()?;
    Ok(digest.finalize().into())
}

fn mix_text(
    digest: &mut Sha256,
    text: &str,
    control: SchemaCompilationControl<'_, '_>,
) -> Result<(), Denial> {
    for chunk in text.as_bytes().chunks(64 * 1024) {
        control.checkpoint()?;
        digest.update(chunk);
    }
    Ok(())
}

#[cfg(test)]
mod tests;
