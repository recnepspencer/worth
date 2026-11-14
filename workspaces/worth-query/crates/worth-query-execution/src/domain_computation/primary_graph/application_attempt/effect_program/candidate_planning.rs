//! Conservative support of one finished existing-record scalar candidate.
use super::*;
use crate::domain_computation::primary_graph::application_checkpoint::facts::FactPayload;
use crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact as Fact;
use worth_query_installation::facade::ApplicationSchema;
use worth_relational::facade::identity::{EntityId, KindId};
#[derive(Clone)]
pub(in crate::domain_computation::primary_graph) struct CandidateFootprint(Arc<Footprint>);
struct Footprint {
    reads: FactPayload,
    writes: Vec<Write>,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
}
#[derive(Eq, PartialEq)]
struct Write {
    entity: EntityId,
    kind: KindId,
    coordinate: Coordinate,
}
#[derive(Eq, PartialEq)]
enum Coordinate {
    Field(AspectFieldLocator),
    Aspect(worth_foundational::facade::AspectKey),
}
impl PartialEq for CandidateFootprint {
    fn eq(&self, other: &Self) -> bool {
        self.0.reads.iter().eq(other.0.reads.iter()) && self.0.writes == other.0.writes
    }
}
impl Eq for CandidateFootprint {}
impl CandidateFootprint {
    // O(read facts * peer scalar coordinates), charged to the original request.
    pub(in crate::domain_computation::primary_graph) fn intersects(
        &self,
        writer: &Self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<bool, WorthQueryApplicationAttemptDenial> {
        let refused = |cause: worth_execution::WorkCeilingDenial| {
            WorthQueryApplicationAttemptDenial::relational_read_stopped(
                cause.into(),
                "authored candidate intersection",
            )
        };
        execution
            .run_native_with_checkpoint(|context| {
                for fact in self.0.reads.iter() {
                    for write in &writer.0.writes {
                        crate::domain_computation::primary_graph::admitted_capture::checkpoint(
                            context, 1,
                        )?;
                        let intersects = match fact {
                            Fact::IndexedEntitySelection {
                                entity_kind,
                                locator,
                                ..
                            } => {
                                *entity_kind == write.kind
                                    && match &write.coordinate {
                                        Coordinate::Field(field) => field == locator,
                                        Coordinate::Aspect(aspect) => {
                                            aspect == locator.aspect().aspect_key()
                                        }
                                    }
                            }
                            _ => fact.touches_entity(write.entity),
                        };
                        if intersects {
                            return Ok(true);
                        }
                    }
                }
                Ok(false)
            })
            .map_err(refused)?
            .0
            .map_err(|cause| refused(worth_execution::WorkCeilingDenial::Stopped(cause)))
    }
}
fn supported(fact: &Fact) -> bool {
    matches!(
        fact,
        Fact::Entity { .. }
            | Fact::Field { .. }
            | Fact::AbsentField { .. }
            | Fact::OptionalAspectAbsent { .. }
            | Fact::AspectPresence { .. }
            | Fact::SourceEntity { .. }
            | Fact::SourceAspectRevision { .. }
            | Fact::SourceFieldRevision { .. }
            | Fact::Relation { .. }
            | Fact::Adjacency { .. }
            | Fact::IndexedEntitySelection { .. }
            | Fact::SourceReadAuthorization { .. }
    )
}
fn kind(facts: &FactPayload, entity: EntityId) -> Option<KindId> {
    facts.iter().find_map(|fact| match fact {
        Fact::Entity { entity_id, kind }
        | Fact::Field {
            entity_id, kind, ..
        }
        | Fact::AbsentField {
            entity_id, kind, ..
        }
        | Fact::OptionalAspectAbsent {
            entity_id, kind, ..
        }
        | Fact::AspectPresence {
            entity_id, kind, ..
        } if *entity_id == entity => Some(*kind),
        _ => None,
    })
}
fn capacity() -> WorthQueryApplicationAttemptDenial {
    denial(
        WorthQueryApplicationAttemptDenialKind::AllocationDenied,
        "authored candidate footprint",
    )
}
fn scalar_write_storage(
    effects: &[WorthQueryApplicationRealizedEffect],
) -> Result<Option<(usize, u64)>, WorthQueryApplicationAttemptDenial> {
    let mut count = 0usize;
    let mut nested_bytes = 0usize;
    for effect in effects {
        let fields = match effect {
            WorthQueryApplicationRealizedEffect::UpdateEntity { fields, .. } => fields.len(),
            WorthQueryApplicationRealizedEffect::PatchOptionalEntityFields { fields, .. } => {
                fields.len()
            }
            WorthQueryApplicationRealizedEffect::RemoveAspect { .. } => 1,
            WorthQueryApplicationRealizedEffect::CreateEntity { .. }
            | WorthQueryApplicationRealizedEffect::DeleteEntity { .. }
            | WorthQueryApplicationRealizedEffect::CreateRelation { .. }
            | WorthQueryApplicationRealizedEffect::DeleteRelation { .. }
            | WorthQueryApplicationRealizedEffect::Emit(_) => return Ok(None),
        };
        count = count.checked_add(fields).ok_or_else(capacity)?;
        let owned = match effect {
            WorthQueryApplicationRealizedEffect::UpdateEntity { fields, .. } => {
                fields.keys().try_fold(0usize, |bytes, field| {
                    bytes.checked_add(field.owned_allocation_capacity_bytes())
                })
            }
            WorthQueryApplicationRealizedEffect::PatchOptionalEntityFields { fields, .. } => {
                fields.keys().try_fold(0usize, |bytes, field| {
                    bytes.checked_add(field.owned_allocation_capacity_bytes())
                })
            }
            WorthQueryApplicationRealizedEffect::RemoveAspect { contract, .. } => {
                Some(contract.key().as_str().len())
            }
            _ => unreachable!("eligible scalar effect"),
        }
        .ok_or_else(capacity)?;
        nested_bytes = nested_bytes.checked_add(owned).ok_or_else(capacity)?;
    }
    let bytes = count
        .checked_mul(size_of::<Write>())
        .and_then(|bytes| bytes.checked_add(size_of::<Footprint>() + 2 * size_of::<usize>()))
        .and_then(|bytes| bytes.checked_add(nested_bytes))
        .and_then(|bytes| u64::try_from(bytes).ok())
        .ok_or_else(capacity)?;
    Ok(Some((count, bytes)))
}
fn copy_scalar_writes(
    facts: &FactPayload,
    effects: &[WorthQueryApplicationRealizedEffect],
    count: usize,
) -> Result<Option<Vec<Write>>, WorthQueryApplicationAttemptDenial> {
    let mut writes = Vec::new();
    writes.try_reserve_exact(count).map_err(|_| capacity())?;
    if writes.capacity() != count {
        return Err(capacity());
    }
    for effect in effects {
        let entity = match effect {
            WorthQueryApplicationRealizedEffect::UpdateEntity { entity_id, .. }
            | WorthQueryApplicationRealizedEffect::PatchOptionalEntityFields {
                entity_id, ..
            }
            | WorthQueryApplicationRealizedEffect::RemoveAspect { entity_id, .. } => *entity_id,
            _ => unreachable!("effect eligibility completed before allocation"),
        };
        let Some(kind) = kind(facts, entity) else {
            return Ok(None);
        };
        match effect {
            WorthQueryApplicationRealizedEffect::UpdateEntity { fields, .. } => {
                for field in fields.keys() {
                    writes.push(Write {
                        entity,
                        kind,
                        coordinate: Coordinate::Field(field.clone()),
                    });
                }
            }
            WorthQueryApplicationRealizedEffect::PatchOptionalEntityFields { fields, .. } => {
                for field in fields.keys() {
                    writes.push(Write {
                        entity,
                        kind,
                        coordinate: Coordinate::Field(field.clone()),
                    });
                }
            }
            WorthQueryApplicationRealizedEffect::RemoveAspect { contract, .. } => {
                writes.push(Write {
                    entity,
                    kind,
                    coordinate: Coordinate::Aspect(contract.key().clone()),
                })
            }
            _ => unreachable!("effect eligibility completed before allocation"),
        }
    }
    Ok(Some(writes))
}
impl<S, O, I, E> WorthQueryApplicationEffectProgram<S, O, I, E> {
    pub(in crate::domain_computation::primary_graph) fn is_admitted_envelope_prefix(
        &self,
        unseen: &crate::domain_computation::primary_graph::application_contribution::authored_edit::UnseenAuthoredEnvelope<'_>,
        binding: crate::domain_computation::primary_graph::WorthQueryApplicationIdempotencyBinding,
    ) -> bool {
        unseen.admits(binding.bind_admission(&self.read_set.admission))
            && self
                .read_set
                .admission
                .graph_work()
                .mutation_product()
                .is_some_and(|product| unseen.coordination.admits(product.observation()))
    }
    pub(in crate::domain_computation::primary_graph) fn candidate_footprint(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Option<CandidateFootprint>, WorthQueryApplicationAttemptDenial> {
        let reads = &self.read_set;
        if !self.authored_stages.is_empty()
            || !self.synchronous_outputs.is_empty()
            || self.conditional_definition.is_some()
            || self.output_currentness_facts.is_some()
            || reads.postcondition_carry.is_some()
            || !self.content_attachments.is_empty()
            || reads.workflow_authority_binding.is_some()
            || reads.workflow_birth_binding.is_some()
            || reads.workflow_succession_binding.is_some()
            || reads.local_action_settlement.is_some()
            || reads.body_external_emission.is_some()
            || reads.signed_creation.is_some()
            || reads.workflow_deadline.is_some()
            || reads.new_commit_refusal.is_some()
            || reads.computation_facts.is_some()
            || !reads.computation_fact_ordinals.is_empty()
            || !reads.consumed_outputs.is_empty()
            || reads.facts.iter().any(|fact| !supported(fact))
        {
            return Ok(None);
        }
        let Some((count, bytes)) = scalar_write_storage(&self.effects)? else {
            return Ok(None);
        };
        let memory = execution
            .allocation_policy()
            .reserve_memory(bytes)
            .map_err(|cause| {
                WorthQueryApplicationAttemptDenial::allocation_denied(
                    "authored candidate footprint",
                    cause,
                )
            })?;
        let Some(writes) = copy_scalar_writes(&reads.facts, &self.effects, count)? else {
            return Ok(None);
        };
        Ok(Some(CandidateFootprint(Arc::new(Footprint {
            reads: reads.facts.clone(),
            writes,
            _memory: memory,
        }))))
    }

    pub(in crate::domain_computation::primary_graph) fn require_planned_support(
        &mut self,
        application: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<S>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        applied: Option<
            &worth_relational::facade::runtime::RelationalPinnedRead<
                worth_relational::facade::runtime::RelationalAppliedCandidateReadIdentity,
            >,
        >,
    ) -> Result<(), WorthQueryApplicationAttemptDenial>
    where
        S: ApplicationSchema,
    {
        if applied.is_some_and(|pin| {
            pin.candidate_identity().proposal().branch_observation()
                != self.read_set.lease.product().relational_basis().reference()
        }) {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::ProjectionAdmissionMismatch,
                "planned authored stage image",
            ));
        }
        self.validate_installed_entry(
            &application.runtime,
            &application.installed_schema.binding_identity(),
        )
        .map_err(|_| {
            denial(
                WorthQueryApplicationAttemptDenialKind::ForeignApplication,
                "planned authored stage",
            )
        })?;
        self.validate_current_authority().map_err(|_| {
            denial(
                WorthQueryApplicationAttemptDenialKind::ForeignApplication,
                "planned authored stage authority",
            )
        })?;
        application
            .runtime
            .primary_provider
            .graph
            .with_runtime(execution, |runtime| {
                for fact in self.read_set.facts.iter() {
                    if !fact.remains_equal_in_image(
                        runtime,
                        execution,
                        self.read_set.lease.snapshot(),
                        applied,
                    )? {
                        return Err(denial(
                            WorthQueryApplicationAttemptDenialKind::MissingAuthoritativeFact,
                            "planned authored stage support changed",
                        ));
                    }
                }
                Ok(())
            })?;
        if let Some(pin) = applied {
            self.read_set.lease.retain_applied_projection(pin.clone());
        }
        Ok(())
    }
}
