//! Reject caller-authored lifecycle records and association changes before native expansion.
use super::{attempt, refused, unavailable, Effect, EntityId, Kind, Lease, Outcome};
use worth_relational::facade::{runtime::ProjectionAspectScope, transactions::EntityReference};

pub(in crate::domain_computation::primary_graph::application_attempt) fn validate_incoming<
    Schema,
>(
    lease: &Lease,
    effects: &[Effect],
    signing: Option<&super::super::signed_creation::SignedCreationState<Schema>>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    source_entity: EntityId,
) -> Result<(), Outcome> {
    let lifecycle = &lease.layout.signing_lifecycle;
    if lifecycle.is_empty() {
        return Ok(());
    }
    for effect in effects {
        let forged = match effect {
            Effect::CreateEntity { kind, .. } => lifecycle.is_record(*kind),
            Effect::CreateRelation { kind, from, to, .. }
                if lifecycle.is_protected_relation(*kind) =>
            {
                if lifecycle.is_consequence(*kind) {
                    true
                } else {
                    !signing.is_some_and(|state| {
                        let Some(Effect::CreateEntity { kind: evidence_kind, key, partition, .. }) = effects.get(state.slot.effect) else { return false; };
                        let EntityReference::Existing(source) = from else { return false; };
                        let EntityReference::Created(target) = to else { return false; };
                        *source == source_entity && target.kind_id == *evidence_kind
                            && target.client_key == worth_relational::facade::symbols::ClientKey::raw(key.clone())
                            && target.partition_id == partition.resolve(worth_relational::facade::identity::PartitionId::main())
                    })
                }
            }
            Effect::DeleteRelation { relation_id } => {
                let kind = lease.handle().with_runtime(execution, |runtime| {
                    runtime
                        .read_truth()
                        .project_snapshot(lease.snapshot())
                        .ok_or_else(unavailable)?
                        .relation_record_with_projection_scope(
                            execution,
                            *relation_id,
                            ProjectionAspectScope::empty(),
                            |record| Some(record.kind_id()),
                        )
                        .map_err(|cause| {
                            attempt(
                                super::super::super::WorthQueryApplicationAttemptDenial::from(
                                    cause,
                                ),
                            )
                        })?
                        .ok_or_else(unavailable)
                })?;
                lifecycle.is_protected_relation(kind)
            }
            _ => false,
        };
        if forged {
            return Err(refused(Kind::SigningLifecycleForgery));
        }
    }
    Ok(())
}
