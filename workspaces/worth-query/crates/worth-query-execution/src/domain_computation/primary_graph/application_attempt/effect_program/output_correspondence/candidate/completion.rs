//! Completion of one candidate's output contract: every exactly-one role and
//! every family minimum is bound, and each binding's posture agrees with the
//! effects the candidate realized. An at-most-one role may stay unbound; its
//! second binding was already refused when it was bound.

use std::collections::BTreeSet;

use worth_relational::facade::transactions::{CreatedEntityRef, EntityReference};

use super::super::WorthQueryApplicationOutputPosture;
use super::{denial, family_matches, WorthQueryApplicationOutputCorrespondenceCandidate};
use crate::domain_computation::primary_graph::application_attempt::effect_program::WorthQueryApplicationRealizedEffect;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
};

impl WorthQueryApplicationOutputCorrespondenceCandidate {
    pub(in crate::domain_computation::primary_graph::application_attempt) fn validate_effects(
        &self,
        effects: &[WorthQueryApplicationRealizedEffect],
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        if let Some((missing, _)) = self.expected_roles.iter().find(|(role, expected)| {
            !expected.cardinality.admits_absence() && !self.roles.contains_key(*role)
        }) {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::MissingOutputRole,
                missing,
            ));
        }
        if let Some(missing) = self.expected_families.iter().find(|family| {
            self.roles
                .keys()
                .filter(|role| family_matches(role, &family.prefix))
                .count()
                < family.minimum
        }) {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::MissingOutputRole,
                &missing.prefix,
            ));
        }
        if self.roles.is_empty() {
            return Ok(());
        }
        let mut deleted_entities = BTreeSet::new();
        let mut created_entities = BTreeSet::new();
        for effect in effects {
            match effect {
                WorthQueryApplicationRealizedEffect::DeleteEntity { entity_id } => {
                    deleted_entities.insert(*entity_id);
                }
                WorthQueryApplicationRealizedEffect::CreateEntity {
                    kind,
                    key,
                    partition,
                    ..
                } => {
                    created_entities.insert(CreatedEntityRef {
                        partition_id: partition
                            .resolve(worth_relational::facade::identity::PartitionId::main()),
                        kind_id: *kind,
                        client_key: worth_relational::facade::symbols::ClientKey::raw(key.clone()),
                    });
                }
                _ => {}
            }
        }
        for (role, binding) in &self.roles {
            let meaning_matches = self.expected_roles.get(role).is_some_and(|expected| {
                expected.posture == binding.posture && expected.entity_name == binding.entity_name
            }) || self.expected_families.iter().any(|family| {
                family_matches(role, &family.prefix)
                    && family.postures.allows(binding.posture)
                    && family.entity_name == binding.entity_name
            });
            if !meaning_matches {
                return Err(denial(
                    WorthQueryApplicationAttemptDenialKind::OutputRoleEntityMismatch,
                    role,
                ));
            }
            let is_deleted = match binding.entity {
                EntityReference::Existing(entity_id) => deleted_entities.contains(&entity_id),
                EntityReference::Created(_) => false,
            };
            let action_matches = match binding.posture {
                WorthQueryApplicationOutputPosture::Preserve => !is_deleted,
                WorthQueryApplicationOutputPosture::Create => {
                    matches!(
                        &binding.entity,
                        EntityReference::Created(created)
                            if created_entities.contains(created)
                    )
                }
                WorthQueryApplicationOutputPosture::Retire => is_deleted,
            };
            if !action_matches {
                return Err(denial(
                    WorthQueryApplicationAttemptDenialKind::OutputRoleActionMismatch,
                    role,
                ));
            }
        }
        Ok(())
    }
}
