//! Native envelope association checks; reconstructive succession lives in history.
use super::payload_coverage::OutputPayloadCoverageFailure as Failure;
use super::*;
use crate::history::data::CodecRetirementClosureStop as Stop;

pub(super) fn coverage_commit_stop(
    failure: Failure,
    envelope: &CanonicalCommitEnvelope,
    id: crate::identity::data::EntityId,
    plan: &crate::schema::data::LoweredAspectContractPlan,
    member_ordinal: usize,
) -> TransactionCommitError {
    match failure {
        Failure::Request(cause) => origin_request(cause),
        Failure::Unsupported(reason) => {
            let mut error = crate::publication::data::PublicationError::new(
                crate::publication::bundle::PublicationStage::BundleAssembly,
                reason.entity_detail(
                    "output has unsupported or unclassified present cells",
                    envelope,
                    id,
                    plan,
                    Some(member_ordinal),
                ),
            );
            error.context = error
                .context
                .with_records(vec![crate::transactions::data::RecordRef::Entity(id)]);
            TransactionCommitError::publication(error)
        }
        Failure::Corrupt(detail) => crate::mvcc::origin_denial(detail),
    }
}

impl CanonicalCommitEnvelope {
    /// Reconstructive callers validate the actual birth payload before any
    /// later Preserve patches, using the same native coverage rule as capture.
    pub(in crate::history::data) fn validate_required_output_entity_create(
        &self,
        id: crate::identity::data::EntityId,
        plan: &crate::schema::data::LoweredAspectContractPlan,
        payload: Option<&worth_foundational::facade::AuthoritativeRecordAspectState>,
        context: &mut worth_execution::MapKernelContext<'_, '_>,
    ) -> Result<(), Stop> {
        payload_coverage::validate_created_output_payload(self, id, plan, payload, context).map_err(
            |failure| match failure {
                Failure::Request(cause) => Stop::Request(cause),
                Failure::Unsupported(reason) => Stop::Payload(reason.entity_detail(
                    "output has unsupported or unclassified birth cells",
                    self,
                    id,
                    plan,
                    None,
                )),
                Failure::Corrupt(detail) => Stop::Payload(detail),
            },
        )
    }

    pub(crate) fn is_unmerged_output_publication_step(&self) -> bool {
        self.commit.parents.len() <= 1
            && self.merge_execution_authority.is_none()
            && self.merge_parent_branches.is_empty()
            && self.merge_base_commits.is_empty()
    }

    pub(crate) fn validate_output_membership_header(
        &self,
    ) -> Result<&CanonicalRequiredOutputOrigin, Stop> {
        let origin = self
            .required_output_origin
            .as_deref()
            .ok_or("present output has no installed producer publication")?;
        if origin.commit != self.commit
            || origin.producer.is_empty()
            || origin.basis.branch_id() != &self.branch_context
            || self
                .branch_cell_checkpoint
                .as_ref()
                .map(|cell| &cell.observation)
                != Some(origin.basis.reference())
            || origin.members.is_empty()
            || origin.members.len() != origin.outputs.len()
            || origin
                .members
                .iter()
                .zip(&origin.outputs)
                .any(|(member, id)| member.id != *id)
            || origin.members.iter().any(|member| {
                use crate::mvcc::RequiredOutputRolePosture as Posture;
                member.role.is_empty()
                    || member.entity_name.is_empty()
                    || member.birth_producer.is_empty()
                    || match origin.publication {
                        CanonicalOutputPublication::Birth
                        | CanonicalOutputPublication::BirthAfterCodecRetirement { .. } => {
                            member.posture != Posture::Create
                                || member.before_write.is_some()
                                || member.birth != self.commit.commit_id
                                || member.birth_producer != origin.producer
                                || member.birth_dependency != origin.dependency_identity
                        }
                        CanonicalOutputPublication::Preserve { .. } => {
                            (member.posture == Posture::Create) != member.before_write.is_none()
                        }
                    }
            })
            || origin.outputs.windows(2).any(|pair| pair[0] >= pair[1])
        {
            return Err("copied or incomplete native output publication membership".into());
        }
        Ok(origin)
    }
}
