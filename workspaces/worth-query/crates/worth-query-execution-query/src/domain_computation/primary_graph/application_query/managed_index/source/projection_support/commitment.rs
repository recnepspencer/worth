//! Commit to the complete physical acquisition, not just provider matches.
use super::*;
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationObservedFact as Fact;
use sha2::{Digest, Sha256};
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;

pub fn capture(
    execution: &ExecutionRequest<'_, '_>,
    facts: &[Fact],
) -> Result<[u8; 32], MaterialDenial> {
    let mut digest = Sha256::new();
    digest.update(b"worth-query:applied-source-acquisition:v1");
    digest.update(
        u64::try_from(facts.len())
            .map_err(|_| Cause::WorkCounterOverflow)?
            .to_be_bytes(),
    );
    for fact in facts {
        SourceFactsAllocation::work(execution, 96)?;
        match fact {
            Fact::SourceEntity { entity_id } => { digest.update([0]); entity(&mut digest, *entity_id); }
            Fact::SourceAspectRevision { entity_id, aspect, native_revision } => {
                digest.update([1]); entity(&mut digest, *entity_id);
                text(execution, &mut digest, aspect.as_str())?;
                revision(&mut digest, *native_revision);
            }
            Fact::SourceFieldRevision { entity_id, locator, native_revision } => {
                digest.update([2]); entity(&mut digest, *entity_id);
                text(execution, &mut digest, locator.aspect().aspect_key().as_str())?;
                digest.update(u64::try_from(locator.field_path().fields().len()).map_err(|_| Cause::WorkCounterOverflow)?.to_be_bytes());
                for field in locator.field_path().fields() { text(execution, &mut digest, field.as_str())?; }
                revision(&mut digest, native_revision.map(|revision| revision.version().0));
                digest.update([native_revision.map_or(0, |revision| match revision.presence() {
                    worth_relational::facade::runtime::RelationalFieldPresence::Present => 1,
                    worth_relational::facade::runtime::RelationalFieldPresence::Absent => 2,
                })]);
            }
            Fact::SourceAdjacencyRevision { anchor, relation_kind, direction, native_revision, .. } => {
                digest.update([3]); entity(&mut digest, *anchor);
                digest.update(relation_kind.as_u32().to_be_bytes());
                digest.update([match direction {
                    worth_relational::facade::runtime::RelationalAdjacencyDirection::Outgoing => 0,
                    worth_relational::facade::runtime::RelationalAdjacencyDirection::Incoming => 1,
                }]);
                revision(&mut digest, native_revision.map(|revision| revision.0));
            }
            _ => return Err(MaterialDenial::Native(
                worth_relational::facade::runtime::ManagedIndexSourcePreparationDenial::ForeignSource)),
        }
    }
    Ok(digest.finalize().into())
}
fn entity(digest: &mut Sha256, entity: worth_relational::facade::identity::EntityId) {
    digest.update(entity.partition_value().to_be_bytes());
    digest.update(entity.local_slot_value().to_be_bytes());
    digest.update(entity.generation_value().to_be_bytes());
}
fn revision(digest: &mut Sha256, value: Option<u64>) {
    digest.update([u8::from(value.is_some())]);
    digest.update(value.unwrap_or_default().to_be_bytes());
}
fn text(
    execution: &ExecutionRequest<'_, '_>,
    digest: &mut Sha256,
    value: &str,
) -> Result<(), MaterialDenial> {
    let length = u64::try_from(value.len()).map_err(|_| Cause::WorkCounterOverflow)?;
    SourceFactsAllocation::work(
        execution,
        length.checked_add(8).ok_or(Cause::WorkCounterOverflow)?,
    )?;
    digest.update(length.to_be_bytes());
    digest.update(value.as_bytes());
    Ok(())
}
