//! Corrupt only a real Query-prepared participant at this Native receiver.
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::required_output_origin::group_selection::CanonicalCommitEnvelopeBodyOwnerPort as _;
use super::*;
use worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasisFieldsPort as _;
impl RelationalAuthoredCoupledCandidate<'_> {
    /// Closed corruption, never an alternate assembly door. Every argument came
    /// through the real installed writer; Native's ordinary admission still runs.
    #[doc(hidden)]
    pub fn append_corrupt_participant_for_test(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        posture: RelationalCoupledProducerPosture,
        content: &crate::content::RelationalContentAttachments,
        mut batches: Vec<crate::transactions::data::primitives::WorkerIntentBatch>,
        fault: u8,
    ) -> Result<(), RelationalCoupledAssemblyFailure> {
        if fault == 4 {
            self.append_publication(execution, posture, content, batches)?;
            return self
                .replay_continuation_probe(execution)
                .map_err(Into::into);
        }
        if matches!(fault, 7..=9) {
            return self.continued_round_probe(execution, posture, content, batches, fault);
        }
        if matches!(fault, 5 | 6) {
            corruption_commands::delete_entity(
                &posture,
                &self.parts,
                &self.mutation.outcome.created_entities,
                self.authored_source(),
                &mut batches,
                fault == 6,
            )?;
            return self.append_publication(execution, posture, content, batches);
        }
        let RelationalCoupledProducerPosture::Publication(mut publication) = posture else {
            return self.append_publication(execution, posture, content, batches);
        };
        if fault == 3 {
            let RequiredOutputPublication::Preserve { predecessor, .. } = &publication else {
                return self.append_publication(
                    execution,
                    RelationalCoupledProducerPosture::Publication(publication),
                    content,
                    batches,
                );
            };
            let registry = (*self.binding.owners.before.owner_inner())
                .root
                .schema_authority()
                .registry();
            corruption_commands::delete_membership(predecessor, registry, &mut batches)?;
        } else if fault == 2 {
            let RequiredOutputPublication::Preserve {
                predecessor,
                meaning,
                roles,
                ..
            } = publication
            else {
                return self.append_publication(
                    execution,
                    RelationalCoupledProducerPosture::Publication(publication),
                    content,
                    batches,
                );
            };
            let foreign = worth_proof::InstalledApplicationPublicationOwner::fresh();
            let admission = foreign.admit_required_output(
                predecessor.origin.producer.clone(),
                *predecessor.origin.dependency_identity.bytes(),
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
                .demand_canonical_envelope(execution, older)
                .map_err(custody::execution_stop)?
                .ok_or_else(missing)?;
            let origin = execution
                .run_native_with_checkpoint(|context| {
                    envelope
                        .validate_output_origins(context)
                        .and_then(|origins| {
                            origins.output_origin_for_route(&predecessor.origin.meaning, context)
                        })
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
        self.append_publication(
            execution,
            RelationalCoupledProducerPosture::Publication(publication),
            content,
            batches,
        )
    }
}
