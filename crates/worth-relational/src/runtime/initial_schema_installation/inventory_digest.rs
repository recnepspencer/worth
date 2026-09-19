use crate::validation::data::CustomInvariantRegistration;

pub fn custom_invariant_inventory_digest(
    registrations: &[CustomInvariantRegistration],
) -> [u8; 32] {
    use sha2::{Digest, Sha256};
    let mut identities = registrations
        .iter()
        .map(|registration| {
            let descriptor = registration.descriptor();
            (
                descriptor.identity.rule_id.as_str().to_owned(),
                descriptor.identity.semantic_version,
                registration.execution_point(),
                descriptor.display_name.to_string(),
                registration.groups().mask(),
                registration.cost_class(),
                registration.failure_effect(),
                registration.maximum_work_units(),
                descriptor.operational.access.clone(),
            )
        })
        .collect::<Vec<_>>();
    identities.sort();
    let mut digest = Sha256::new();
    for (rule, version, point, display_name, groups, cost, failure, work, access) in identities {
        digest.update((rule.len() as u64).to_le_bytes());
        digest.update(rule.as_bytes());
        digest.update(version.major.to_le_bytes());
        digest.update(version.minor.to_le_bytes());
        digest.update([point as u8]);
        digest.update((display_name.len() as u64).to_le_bytes());
        digest.update(display_name.as_bytes());
        digest.update(groups.to_le_bytes());
        digest.update([cost as u8, failure as u8]);
        digest.update(work.get().to_le_bytes());
        for kinds in [
            &access.read_entity_kinds,
            &access.read_relation_kinds,
            &access.affected_entity_kinds,
            &access.affected_relation_kinds,
        ] {
            digest.update((kinds.len() as u64).to_le_bytes());
            for kind in kinds {
                digest.update(kind.0.to_le_bytes());
            }
        }
    }
    digest.finalize().into()
}
