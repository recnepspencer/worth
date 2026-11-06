//! Admit canonical-role/closure separation before the native World transition.
use super::*;
impl WorthQueryApplicationOutputCorrespondenceCandidate {
    pub(in crate::domain_computation::primary_graph) fn admit_primary_publication<'a>(
        &mut self,
        members: impl ExactSizeIterator<
            Item = (
                &'a str,
                &'a str,
                worth_relational::facade::mvcc::RequiredOutputRolePosture,
                EntityId,
            ),
        >,
    ) -> Result<
        (),
        (
            super::super::WorthQueryApplicationOutputProjectionDenial,
            String,
        ),
    > {
        use super::super::WorthQueryApplicationOutputProjectionDenial as Denial;
        use worth_relational::facade::mvcc::RequiredOutputRolePosture as Native;
        let mut canonical = std::collections::BTreeSet::new();
        for (role, entity, posture, id) in members {
            let binding = self
                .roles
                .get(role)
                .ok_or_else(|| (Denial::MissingRole, role.to_owned()))?;
            if binding.entity_name != entity
                || matches!(&binding.entity,EntityReference::Existing(existing) if *existing!=id)
            {
                return Err((Denial::EntityMismatch, role.to_owned()));
            }
            let expected = match posture {
                Native::Create => WorthQueryApplicationOutputPosture::Create,
                Native::Preserve => WorthQueryApplicationOutputPosture::Preserve,
                Native::Retire => WorthQueryApplicationOutputPosture::Retire,
            };
            if binding.posture != expected {
                return Err((Denial::ActionMismatch, role.to_owned()));
            }
            canonical.insert(role);
        }
        // Equal counts do not prove equal membership. Validate every candidate
        // before moving anything, so a typed denial leaves the candidate intact.
        for (role, binding) in &self.roles {
            if !canonical.contains(role.as_str()) {
                if binding.posture != WorthQueryApplicationOutputPosture::Retire {
                    return Err((Denial::ActionMismatch, role.to_owned()));
                }
                if !matches!(binding.entity, EntityReference::Existing(_)) {
                    return Err((Denial::EntityMismatch, role.to_owned()));
                }
            }
        }
        let closure = self
            .roles
            .keys()
            .filter(|role| !canonical.contains(role.as_str()))
            .cloned()
            .collect::<Vec<_>>();
        for role in closure {
            if let Some(binding) = self.roles.remove(&role) {
                let EntityReference::Existing(entity) = binding.entity else {
                    return Err((Denial::EntityMismatch, role));
                };
                self.closure_roles.insert(
                    role,
                    CandidateClosureBinding {
                        entity_name: binding.entity_name,
                        entity_type: binding.entity_type,
                        entity,
                    },
                );
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
