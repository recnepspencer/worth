mod authoring;
mod contract;

use std::any::TypeId;
use std::collections::{BTreeMap, BTreeSet};

#[cfg(test)]
use worth_query_declaration::facade::application_operation::ApplicationMutationOutputPostureSet;
use worth_relational::facade::identity::EntityId;
use worth_relational::facade::transactions::{CommitResult, CreatedEntityRef, EntityReference};

use contract::{ExpectedOutputBinding, ExpectedOutputFamily};

use super::{
    CommittedOutputBinding, WorthQueryApplicationOutputAction,
    WorthQueryApplicationOutputCorrespondence, WorthQueryApplicationOutputPosture,
    WorthQueryApplicationOutputRole,
};
use crate::domain_computation::primary_graph::application_attempt::effect_program::{
    WorthQueryApplicationEffectEntity, WorthQueryApplicationRealizedEffect,
};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
};
#[derive(Clone, Debug, Eq, PartialEq)]
struct CandidateOutputBinding {
    posture: WorthQueryApplicationOutputPosture,
    entity_name: String,
    entity_type: TypeId,
    entity: EntityReference,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph::application_attempt) struct WorthQueryApplicationOutputCorrespondenceCandidate
{
    binding_type: Option<TypeId>,
    expected_roles: BTreeMap<String, ExpectedOutputBinding>,
    expected_families: Vec<ExpectedOutputFamily>,
    roles: BTreeMap<String, CandidateOutputBinding>,
}

impl WorthQueryApplicationOutputCorrespondenceCandidate {
    pub(in crate::domain_computation::primary_graph::application_attempt) fn is_empty(
        &self,
    ) -> bool {
        self.expected_roles.is_empty() && self.expected_families.is_empty() && self.roles.is_empty()
    }

    pub(in crate::domain_computation::primary_graph::application_attempt) fn remap_created_partition(
        mut self,
        mutation_partition: worth_relational::facade::identity::PartitionId,
    ) -> Self {
        for binding in self.roles.values_mut() {
            if let EntityReference::Created(created) = &mut binding.entity {
                created.partition_id = mutation_partition;
            }
        }
        self
    }

    #[cfg(test)]
    pub(super) fn prepare_test_role<Binding, Entity, Action>(
        &mut self,
        role: WorthQueryApplicationOutputRole<Binding, Entity, Action>,
        entity_name: &'static str,
    ) where
        Binding: 'static,
        Entity: 'static,
        Action: WorthQueryApplicationOutputAction,
    {
        self.binding_type = Some(TypeId::of::<Binding>());
        self.expected_roles.insert(
            role.name().to_owned(),
            ExpectedOutputBinding {
                posture: Action::POSTURE,
                entity_name,
            },
        );
    }

    #[cfg(test)]
    pub(super) fn prepare_test_family<Binding>(
        &mut self,
        prefix: &str,
        postures: ApplicationMutationOutputPostureSet,
        entity_name: &'static str,
        minimum: usize,
    ) where
        Binding: 'static,
    {
        self.binding_type = Some(TypeId::of::<Binding>());
        self.expected_families.push(ExpectedOutputFamily {
            prefix: prefix.to_owned(),
            postures,
            entity_name,
            minimum,
        });
    }

    fn validate_binding<Schema, Binding, Entity, Action>(
        &self,
        role: &WorthQueryApplicationOutputRole<Binding, Entity, Action>,
        target: &WorthQueryApplicationEffectEntity<Schema, Entity>,
        program: &std::sync::Arc<()>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial>
    where
        Binding: 'static,
        Entity: 'static,
        Action: WorthQueryApplicationOutputAction,
    {
        validate_role_name(role.name())?;
        if !std::sync::Arc::ptr_eq(program, &target.program) {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::ForeignEffectTarget,
                role.name(),
            ));
        }
        validate_reference_posture(Action::POSTURE, &target.reference, role.name())?;
        let binding_type = TypeId::of::<Binding>();
        if self
            .binding_type
            .is_some_and(|existing| existing != binding_type)
        {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::ForeignOutputRole,
                role.name(),
            ));
        }
        let exact = self.expected_roles.get(role.name());
        let family = self
            .expected_families
            .iter()
            .find(|family| family_matches(role.name(), &family.prefix));
        let (posture_allowed, expected_entity) = match (exact, family) {
            (Some(expected), None) => (expected.posture == Action::POSTURE, expected.entity_name),
            (None, Some(expected)) => (
                expected.postures.allows(Action::POSTURE),
                expected.entity_name,
            ),
            _ => {
                return Err(denial(
                    WorthQueryApplicationAttemptDenialKind::UndeclaredOutputRole,
                    role.name(),
                ))
            }
        };
        if expected_entity != target.entity {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::OutputRoleEntityMismatch,
                role.name(),
            ));
        }
        if !posture_allowed {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::OutputRoleActionMismatch,
                role.name(),
            ));
        }
        if self.roles.contains_key(role.name()) {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::DuplicateOutputRole,
                role.name(),
            ));
        }
        Ok(())
    }

    fn insert_binding<Schema, Binding, Entity, Action>(
        &mut self,
        role: WorthQueryApplicationOutputRole<Binding, Entity, Action>,
        target: &WorthQueryApplicationEffectEntity<Schema, Entity>,
    ) where
        Binding: 'static,
        Entity: 'static,
        Action: WorthQueryApplicationOutputAction,
    {
        self.binding_type = Some(TypeId::of::<Binding>());
        self.roles.insert(
            role.name().to_owned(),
            CandidateOutputBinding {
                posture: Action::POSTURE,
                entity_name: target.entity.clone(),
                entity_type: TypeId::of::<Entity>(),
                entity: target.reference.clone(),
            },
        );
    }

    #[cfg(test)]
    pub(super) fn bind<Schema, Binding, Entity, Action>(
        &mut self,
        role: WorthQueryApplicationOutputRole<Binding, Entity, Action>,
        target: &WorthQueryApplicationEffectEntity<Schema, Entity>,
        program: &std::sync::Arc<()>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial>
    where
        Binding: 'static,
        Entity: 'static,
        Action: WorthQueryApplicationOutputAction,
    {
        self.validate_binding(&role, target, program)?;
        self.insert_binding(role, target);
        Ok(())
    }

    pub(in crate::domain_computation::primary_graph::application_attempt) fn validate_effects(
        &self,
        effects: &[WorthQueryApplicationRealizedEffect],
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        if let Some(missing) = self
            .expected_roles
            .keys()
            .find(|role| !self.roles.contains_key(*role))
        {
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

    pub(in crate::domain_computation::primary_graph) fn seal(
        self,
        commit: &CommitResult,
    ) -> WorthQueryApplicationOutputCorrespondence {
        self.seal_with(|created| commit.created_entity(created))
    }

    pub(super) fn seal_with(
        self,
        resolve_created: impl Fn(
            &worth_relational::facade::transactions::CreatedEntityRef,
        ) -> Option<EntityId>,
    ) -> WorthQueryApplicationOutputCorrespondence {
        let roles = self
            .roles
            .into_iter()
            .map(|(role, binding)| {
                let entity = match binding.entity {
                    EntityReference::Existing(entity_id) => entity_id,
                    EntityReference::Created(created) => resolve_created(&created).expect(
                        "a performed Relational commit resolves every admitted created output",
                    ),
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
            })
            .collect();
        WorthQueryApplicationOutputCorrespondence {
            binding_type: self.binding_type,
            roles,
        }
    }
}

fn validate_reference_posture(
    posture: WorthQueryApplicationOutputPosture,
    reference: &EntityReference,
    role: &str,
) -> Result<(), WorthQueryApplicationAttemptDenial> {
    let valid = matches!(
        (posture, reference),
        (
            WorthQueryApplicationOutputPosture::Create,
            EntityReference::Created(_)
        ) | (
            WorthQueryApplicationOutputPosture::Preserve,
            EntityReference::Existing(_)
        ) | (
            WorthQueryApplicationOutputPosture::Retire,
            EntityReference::Existing(_)
        )
    );
    valid.then_some(()).ok_or_else(|| {
        denial(
            WorthQueryApplicationAttemptDenialKind::OutputRoleActionMismatch,
            role,
        )
    })
}

fn validate_role_name(role: &str) -> Result<(), WorthQueryApplicationAttemptDenial> {
    if super::role::validate_output_role_name(role).is_err() {
        Err(denial(
            WorthQueryApplicationAttemptDenialKind::InvalidOutputRole,
            role,
        ))
    } else {
        Ok(())
    }
}

fn family_matches(role: &str, prefix: &str) -> bool {
    role.strip_prefix(prefix)
        .is_some_and(|member| !member.is_empty())
}

fn denial(
    kind: WorthQueryApplicationAttemptDenialKind,
    subject: impl Into<String>,
) -> WorthQueryApplicationAttemptDenial {
    WorthQueryApplicationAttemptDenial::new(kind, subject)
}
