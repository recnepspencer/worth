//! Canonical selected answer and dependency content, without source custody.
use super::*;
use sha2::{Digest, Sha256};
pub(super) fn derive<E>(
    key: &ComponentSelectionKey,
    aggregates: &[Arc<AggregateObservation>],
    facts: &[SourceFacts],
    mut poll: impl FnMut() -> Result<(), E>,
) -> Result<[u8; 32], E> {
    poll()?;
    let mut digest = Sha256::new();
    digest.update(b"worth-query:component-selection:v1");
    digest.update(key.locator().as_bytes());
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

pub(super) fn derive_members<E>(
    key: &ComponentSelectionKey,
    aggregates: &[Arc<AggregateObservation<worth_relational::facade::schema::NativeMemberKey>>],
    facts: &[SourceFacts],
    mut poll: impl FnMut() -> Result<(), E>,
) -> Result<[u8; 32], E> {
    poll()?;
    let mut digest = Sha256::new();
    digest.update(b"worth-query:derived-component-selection:v1");
    key.checkpoint_subject(&mut poll)?;
    digest.update(key.locator().as_bytes());
    digest.update((aggregates.len() as u64).to_be_bytes());
    for body in aggregates {
        poll()?;
        digest.update(body.native().definition().bytes());
        digest.update(body.native().identity().bytes());
        digest.update((body.native().len() as u64).to_be_bytes());
        for member in body.native().members() {
            poll()?;
            crate::domain_computation::primary_graph::aggregate_observation::member_work::checkpoint_names(member,&mut poll)?;
            let mut text = String::new();
            key::write_member(&mut text, member).expect("String write");
            digest.update((text.len() as u64).to_be_bytes());
            digest.update(text.as_bytes());
        }
    }
    digest.update((facts.len() as u64).to_be_bytes());
    for fact in facts {
        fact.append_descriptor_with_poll(&mut digest, &mut poll)?;
    }
    poll()?;
    Ok(digest.finalize().into())
}
