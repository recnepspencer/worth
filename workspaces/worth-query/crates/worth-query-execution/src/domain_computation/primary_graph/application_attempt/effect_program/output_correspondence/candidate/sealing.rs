//! Seal already admitted canonical roles and separately typed closure evidence.
use super::*;
impl WorthQueryApplicationOutputCorrespondenceCandidate {
    pub(in crate::domain_computation::primary_graph) fn seal(
        self,
        commit: &CommitResult,
    ) -> WorthQueryApplicationOutputCorrespondence {
        self.seal_with(|created| commit.created_entity(created))
    }

    pub(super) fn seal_with(
        mut self,
        resolve_created: impl Fn(
            &worth_relational::facade::transactions::CreatedEntityRef,
        ) -> Option<EntityId>,
    ) -> WorthQueryApplicationOutputCorrespondence {
        let resolve = |(role, binding): (String, CandidateOutputBinding)| {
            let entity = match binding.entity {
                EntityReference::Existing(id) => id,
                EntityReference::Created(created) => resolve_created(&created)
                    .expect("a performed Relational commit resolves every admitted created output"),
            };
            (
                role,
                CommittedOutputBinding {
                    posture: binding.posture,
                    entity_name: binding.entity_name,
                    entity_type: binding.entity_type,
                    entity,
                },
            )
        };
        let roles = std::mem::take(&mut self.roles)
            .into_iter()
            .map(&resolve)
            .collect();
        let closure = std::mem::take(&mut self.closure_roles)
            .into_iter()
            .map(|(role, binding)| (role, binding.committed()))
            .collect();
        self.finish_resolved_roles(roles, closure)
    }

    /// Native has performed this exact participant's original typed roles. Its
    /// canonical member identities resolve local Create refs independently of
    /// other writers in the same atomic transaction.
    pub(in crate::domain_computation::primary_graph) fn seal_from_publication(
        mut self,
        publication: &worth_relational::facade::mvcc::RelationalRequiredOutputPublication,
    ) -> WorthQueryApplicationOutputCorrespondence {
        let mut resolved = Vec::with_capacity(self.roles.len());
        for (role, entity_name, _, entity) in publication.members() {
            let (role, binding) = self
                .roles
                .remove_entry(role)
                .expect("Native performed every role of this registered participant");
            assert_eq!(
                binding.entity_name, entity_name,
                "Native performed the registered entity contract"
            );
            resolved.push((
                role,
                CommittedOutputBinding {
                    posture: binding.posture,
                    entity_name: binding.entity_name,
                    entity_type: binding.entity_type,
                    entity,
                },
            ));
        }
        assert!(
            self.roles.is_empty(),
            "Native performed the complete registered role set"
        );
        resolved.sort_unstable_by(|left, right| left.0.cmp(&right.0));
        let closure = std::mem::take(&mut self.closure_roles)
            .into_iter()
            .map(|(role, binding)| (role, binding.committed()))
            .collect();
        self.finish_resolved_roles(resolved, closure)
    }
    fn finish_resolved_roles(
        self,
        roles: Vec<(String, CommittedOutputBinding)>,
        closure_evidence: Vec<(String, CommittedOutputBinding)>,
    ) -> WorthQueryApplicationOutputCorrespondence {
        let optional_roles = self
            .expected_roles
            .into_iter()
            .filter(|(_, expected)| expected.cardinality.admits_absence())
            .map(|(role, _)| role)
            .collect();
        let mut storage =
            super::super::CorrespondenceStorage::from_committed(optional_roles, roles);
        storage.closure_evidence = closure_evidence;
        WorthQueryApplicationOutputCorrespondence {
            binding_type: self.binding_type,
            contract_type: self.contract_type,
            storage: std::sync::Arc::new(storage),
        }
    }
}
