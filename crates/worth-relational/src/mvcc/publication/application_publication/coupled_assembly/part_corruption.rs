//! Corrupt only a real Query-prepared participant at this Native receiver.
use super::*;
impl RelationalAuthoredCoupledCandidate<'_> {
    /// Closed corruption, never an alternate assembly door. Every argument came
    /// through the real installed writer; Native's ordinary admission still runs.
    #[doc(hidden)]
    pub fn append_corrupt_participant_for_test(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        mut publication: RequiredOutputPublication,
        content: &crate::content::RelationalContentAttachments,
        mut batches: Vec<crate::transactions::data::WorkerIntentBatch>,
        fault: u8,
    ) -> Result<(), RelationalCoupledAssemblyFailure> {
        if fault == 3 {
            let RequiredOutputPublication::Preserve { predecessor, .. } = &publication else {
                return self.append_preserve(execution, publication, content, batches);
            };
            let registry = self
                .binding
                .owners
                .before
                .inner
                .root
                .schema_authority()
                .registry();
            let edge = predecessor
                .origin
                .relations
                .iter()
                .find(|edge| {
                    registry.relation_kinds.get(&edge.kind).is_some_and(|kind| {
                        kind.kind_name.starts_with("worth-aggregate-membership-")
                    })
                })
                .ok_or_else(|| {
                    part::reject(
                        crate::publication::data::AuthoredRetirementRefusal::CorruptPublication,
                        predecessor.origin.meaning.source_scope,
                    )
                })?;
            let Some(batch) = batches.first_mut() else {
                return Err(
                    crate::mvcc::origin_denial("real coupled writer has no command batch").into(),
                );
            };
            batch
                .intents
                .push(crate::transactions::data::MutationIntent::Relation(
                    crate::transactions::data::RelationMutationIntent::Delete(
                        crate::transactions::data::DeleteRelationIntent {
                            relation_id: edge.id,
                        },
                    ),
                ));
        } else if fault == 2 {
            let RequiredOutputPublication::Preserve {
                predecessor,
                meaning,
                roles,
                ..
            } = publication
            else {
                return self.append_preserve(execution, publication, content, batches);
            };
            let foreign = worth_proof::InstalledApplicationPublicationOwner::fresh();
            let admission = foreign.admit_required_output(
                predecessor.origin.producer.clone(),
                predecessor.origin.dependency_identity,
            );
            publication = RequiredOutputPublication::Birth {
                admission,
                absence: None,
                meaning,
                roles,
            };
        } else if let RequiredOutputPublication::Preserve { predecessor, .. } = &mut publication {
            let subject = predecessor.origin.meaning.source_scope;
            let missing = || {
                part::reject(
                    crate::publication::data::AuthoredRetirementRefusal::CorruptPublication,
                    subject,
                )
            };
            let older = predecessor
                .origin
                .publication
                .predecessor()
                .ok_or_else(missing)?;
            let envelope = self
                .runtime
                .history
                .canonical_envelope(older)
                .ok_or_else(missing)?;
            let origin = execution
                .run_native_with_checkpoint(|context| {
                    envelope
                        .output_origin_for_route(&predecessor.origin.meaning, context)
                        .map(Arc::clone)
                        .map_err(|stop| match stop {
                            crate::history::data::CodecRetirementClosureStop::Request(cause) => {
                                custody::execution_stop(cause)
                            }
                            _ => missing(),
                        })
                })
                .map_err(custody::execution_stop)?
                .0?;
            predecessor.origin = origin;
            predecessor.evidence =
                crate::publication::ownership::PublicationEvidence::Canonical(envelope);
        }
        self.append_preserve(execution, publication, content, batches)
    }
}
