//! Native envelope association checks; reconstructive succession lives in history.
use super::payload_coverage::OutputPayloadCoverageFailure as Failure;
use super::*;
use crate::history::data::CodecRetirementClosureStop as Stop;

pub(super) fn coverage_commit_stop(failure: Failure) -> TransactionCommitError {
    match failure {
        Failure::Request(cause) => origin_request(cause),
        Failure::Unsupported => {
            crate::mvcc::origin_denial("output has unsupported or unclassified present cells")
        }
        Failure::Corrupt(detail) => crate::mvcc::origin_denial(detail),
    }
}

impl CanonicalCommitEnvelope {
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
