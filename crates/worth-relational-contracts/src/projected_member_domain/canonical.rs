//! Optional canonical domain suffix; old authorities receive no new bytes.
use super::ProjectedMemberDomainParts;
use sha2::{Digest, Sha256};

pub fn update_digest(hasher: &mut Sha256, domains: &[ProjectedMemberDomainParts]) {
    update_digest_for_compilation(
        hasher,
        domains,
        crate::schema::compilation_control::SchemaCompilationControl::Configuration,
    )
    .expect("configuration domain digest has no request refusal")
}
pub fn update_digest_for_compilation(
    hasher: &mut Sha256,
    domains: &[ProjectedMemberDomainParts],
    control: crate::schema::compilation_control::SchemaCompilationControl<'_, '_>,
) -> Result<(), crate::execution::denial_cause::RelationalExecutionDenialCause> {
    if domains.is_empty() {
        return Ok(());
    }
    hasher.update(b"\0WORTH_PROJECTED_MEMBER_DOMAINS_V1\0");
    hasher.update((domains.len() as u64).to_le_bytes());
    for domain in domains {
        control.checkpoint()?;
        for text in [&domain.identity.schema.0, &domain.identity.name] {
            text_bytes(hasher, text);
        }
        hasher.update(domain.identity.version.0.to_le_bytes());
        hasher.update(domain.carrier.0.to_le_bytes());
        hasher.update((domain.cases.len() as u64).to_le_bytes());
        for case in &domain.cases {
            control.checkpoint()?;
            text_bytes(hasher, case);
        }
    }
    Ok(())
}

fn text_bytes(hasher: &mut Sha256, text: &str) {
    hasher.update((text.len() as u64).to_le_bytes());
    hasher.update(text.as_bytes());
}
