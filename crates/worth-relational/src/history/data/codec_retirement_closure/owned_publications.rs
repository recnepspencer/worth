//! Projection of Native-admitted complete codec publications.
use super::*;
impl OwnedCodecRetirementClosure {
    pub(crate) fn complete_publications(
        &self,
        context: &mut worth_execution::MapKernelContext<'_, '_>,
        lease: Option<&worth_execution::ExecutionResourceLease<'_>>,
    ) -> Result<
        (
            Vec<
                std::sync::Arc<
                    crate::history::data::canonical_commit_envelope::CanonicalRequiredOutputOrigin,
                >,
            >,
            worth_execution::ExecutionMemoryReservation,
        ),
        CodecRetirementClosureStop,
    > {
        let memory = reserve(
            lease,
            self.origins
                .len()
                .checked_mul(std::mem::size_of::<
                    std::sync::Arc<crate::history::data::canonical_commit_envelope::CanonicalRequiredOutputOrigin>,
                >())
                .ok_or_else(memory_overflow)?,
        )?;
        let mut complete = Vec::with_capacity(self.origins.len());
        for (_, _, origin) in &self.origins {
            charge(
                context,
                complete.len().saturating_add(
                    origin
                        .members
                        .len()
                        .saturating_mul(self.entities.len().saturating_add(1)),
                ),
            )?;
            if complete.iter().any(
                |prior: &std::sync::Arc<
                    crate::history::data::canonical_commit_envelope::CanonicalRequiredOutputOrigin,
                >| prior.commit == origin.commit,
            ) {
                continue;
            }
            if origin.members.iter().all(|member| {
                member.posture == crate::mvcc::RequiredOutputRolePosture::Retire
                    || self.entities.binary_search(&member.id).is_ok()
            }) {
                complete.push(std::sync::Arc::clone(origin));
            }
        }
        Ok((complete, memory))
    }
    pub(crate) fn births(
        &self,
    ) -> impl Iterator<
        Item = (
            EntityId,
            KindId,
            &str,
            &[u8; 32],
            crate::mvcc::RequiredOutputRolePosture,
            crate::mvcc::RequiredOutputRolePosture,
        ),
    > {
        self.origins.iter().map(|(id, kind, origin)| {
            let member = &origin.members[origin
                .members
                .binary_search_by_key(id, |member| member.id)
                .expect(/* Proof: The only origins.push is preceded by membership::validate_member, which checks this exact id in validate_output_membership_header's sorted immutable members. OwnedCodecRetirementClosure fields are private and neither the id nor the Arc origin is mutated afterwards. */ "readmitted full membership")];
            (
                *id,
                *kind,
                member.birth_producer.as_str(),
                &member.birth_dependency,
                member.posture,
                origin.publication.predecessor().map_or(
                    crate::mvcc::RequiredOutputRolePosture::Create,
                    |_| crate::mvcc::RequiredOutputRolePosture::Preserve,
                ),
            )
        })
    }
    pub(crate) fn producers(&self) -> impl Iterator<Item = (EntityId, KindId, &str, &[u8; 32])> {
        self.origins.iter().map(|(id, kind, origin)| {
            (
                *id,
                *kind,
                origin.producer.as_str(),
                &origin.dependency_identity,
            )
        })
    }
}
