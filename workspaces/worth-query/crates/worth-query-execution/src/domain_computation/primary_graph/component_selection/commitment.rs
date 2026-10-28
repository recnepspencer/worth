//! Canonical selected answer and dependency content, without source custody.
use super::*;
use sha2::{Digest, Sha256};
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;
pub(super) fn derive(
    key: &ComponentSelectionKey,
    aggregates: &[Arc<AggregateObservation>],
    facts: &[SourceFacts],
    mut poll: impl FnMut() -> Result<(), Cause>,
) -> Result<[u8; 32], Cause> {
    poll()?;
    let mut digest = Sha256::new();
    digest.update(b"worth-query:component-selection:v1");
    key.write_locator(&mut DigestText(&mut digest))
        .map_err(|_| Cause::ChargedBytesOverflow)?;
    digest.update((aggregates.len() as u64).to_be_bytes());
    for body in aggregates {
        poll()?;
        digest.update(body.native().definition().bytes());
        digest.update(body.native().identity().bytes());
        digest.update((body.native().len() as u64).to_be_bytes());
        for entity in body.native().members() {
            poll()?;
            digest.update(entity.partition_value().to_be_bytes());
            digest.update(entity.local_slot_value().to_be_bytes());
            digest.update(entity.generation_value().to_be_bytes());
        }
    }
    digest.update((facts.len() as u64).to_be_bytes());
    for fact in facts {
        fact.append_descriptor_with_poll(&mut digest, &mut poll)?;
    }
    poll()?;
    Ok(digest.finalize().into())
}

pub(super) fn derive_members(
    key: &ComponentSelectionKey,
    aggregates: &[Arc<AggregateObservation<worth_relational::facade::schema::NativeMemberKey>>],
    facts: &[SourceFacts],
    mut poll: impl FnMut() -> Result<(), Cause>,
) -> Result<[u8; 32], Cause> {
    poll()?;
    let mut digest = Sha256::new();
    digest.update(b"worth-query:derived-component-selection:v1");
    key.checkpoint_subject(&mut poll)?;
    key.write_locator(&mut DigestText(&mut digest))
        .map_err(|_| Cause::ChargedBytesOverflow)?;
    digest.update((aggregates.len() as u64).to_be_bytes());
    for body in aggregates {
        poll()?;
        digest.update(body.native().definition().bytes());
        digest.update(body.native().identity().bytes());
        digest.update((body.native().len() as u64).to_be_bytes());
        for member in body.native().members() {
            poll()?;
            crate::domain_computation::primary_graph::aggregate_observation::member_work::checkpoint_names(member,&mut poll)?;
            let mut length = TextLength(0);
            key::write_member(&mut length, member).map_err(|_| Cause::ChargedBytesOverflow)?;
            digest.update(length.0.to_be_bytes());
            key::write_member(&mut DigestText(&mut digest), member)
                .map_err(|_| Cause::ChargedBytesOverflow)?;
        }
    }
    digest.update((facts.len() as u64).to_be_bytes());
    for fact in facts {
        fact.append_descriptor_with_poll(&mut digest, &mut poll)?;
    }
    poll()?;
    Ok(digest.finalize().into())
}

// These sinks share the actual locator/member emitter. The existing semantic
// checkpoint covers the count and digest passes; neither sink owns text backing.
struct TextLength(u64);
impl std::fmt::Write for TextLength {
    fn write_str(&mut self, text: &str) -> std::fmt::Result {
        self.0 = self
            .0
            .checked_add(u64::try_from(text.len()).map_err(|_| std::fmt::Error)?)
            .ok_or(std::fmt::Error)?;
        Ok(())
    }
}
struct DigestText<'a>(&'a mut Sha256);
impl std::fmt::Write for DigestText<'_> {
    fn write_str(&mut self, text: &str) -> std::fmt::Result {
        self.0.update(text.as_bytes());
        Ok(())
    }
}
