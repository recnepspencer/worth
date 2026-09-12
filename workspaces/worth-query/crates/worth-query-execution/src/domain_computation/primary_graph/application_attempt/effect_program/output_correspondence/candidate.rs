mod authoring;

use std::any::TypeId;
use std::collections::BTreeMap;

use worth_query_declaration::facade::application_operation::{
    ApplicationMutationBinding, ApplicationMutationOutputContract,
};
use worth_query_installation::facade::ApplicationSchema;
use worth_relational::facade::identity::EntityId;
use worth_relational::facade::transactions::{CommitResult, EntityReference};

use super::{
    action, CommittedOutputBinding, WorthQueryApplicationOutputCorrespondence,
    WorthQueryApplicationOutputPosture, WorthQueryApplicationOutputRole,
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
    entity: EntityReference,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ExpectedOutputBinding {
    posture: WorthQueryApplicationOutputPosture,
    entity_name: &'static str,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph::application_attempt) struct WorthQueryApplicationOutputCorrespondenceCandidate
{
    binding_type: Option<TypeId>,
    expected_roles: BTreeMap<String, ExpectedOutputBinding>,
    roles: BTreeMap<String, CandidateOutputBinding>,
}

impl WorthQueryApplicationOutputCorrespondenceCandidate {
    #[cfg(test)]
    pub(super) fn prepare_test_role<Binding, Entity, Action>(
        &mut self,
        role: WorthQueryApplicationOutputRole<Binding, Entity, Action>,
        entity_name: &'static str,
    ) where
        Binding: 'static,
        Action: action::Sealed,
    {
        self.binding_type = Some(TypeId::of::<Binding>());
        self.expected_roles.insert(
            role.name.to_owned(),
            ExpectedOutputBinding {
                posture: Action::POSTURE,
                entity_name,
            },
        );
    }

    fn prepare_contract<Schema, Binding>(
        &self,
    ) -> Result<
        (TypeId, BTreeMap<String, ExpectedOutputBinding>, usize),
        WorthQueryApplicationAttemptDenial,
    >
    where
        Schema: ApplicationSchema,
        Binding: ApplicationMutationBinding<Schema>,
    {
        let mut prepared = BTreeMap::new();
        let mut retained_representation_bytes = 0_usize;
        for descriptor in <Binding::Output as ApplicationMutationOutputContract<Schema>>::ROLES {
            validate_role_name(descriptor.name())?;
            if self.expected_roles.contains_key(descriptor.name())
                || prepared
                    .insert(
                        descriptor.name().to_owned(),
                        ExpectedOutputBinding {
                            posture: descriptor.posture(),
                            entity_name: descriptor.entity(),
                        },
                    )
                    .is_some()
            {
                return Err(denial(
                    WorthQueryApplicationAttemptDenialKind::DuplicateOutputRole,
                    descriptor.name(),
                ));
            }
            retained_representation_bytes = retained_representation_bytes
                .checked_add(descriptor.name().len())
                .ok_or_else(|| {
                    denial(
                        WorthQueryApplicationAttemptDenialKind::CandidateReservationExceeded,
                        descriptor.name(),
                    )
                })?;
        }
        Ok((
            TypeId::of::<Binding>(),
            prepared,
            retained_representation_bytes,
        ))
    }

    fn validate_binding<Schema, Binding, Entity, Action>(
        &self,
        role: WorthQueryApplicationOutputRole<Binding, Entity, Action>,
        target: &WorthQueryApplicationEffectEntity<Schema, Entity>,
        program: &std::sync::Arc<()>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial>
    where
        Binding: 'static,
        Action: action::Sealed,
    {
        validate_role_name(role.name)?;
        if !std::sync::Arc::ptr_eq(program, &target.program) {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::ForeignEffectTarget,
                role.name,
            ));
        }
        validate_reference_posture(Action::POSTURE, &target.reference, role.name)?;
        let binding_type = TypeId::of::<Binding>();
        if self
            .binding_type
            .is_some_and(|existing| existing != binding_type)
        {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::ForeignOutputRole,
                role.name,
            ));
        }
        let expected = self.expected_roles.get(role.name).ok_or_else(|| {
            denial(
                WorthQueryApplicationAttemptDenialKind::UndeclaredOutputRole,
                role.name,
            )
        })?;
        if expected.posture != Action::POSTURE || expected.entity_name != target.entity {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::OutputRoleEntityMismatch,
                role.name,
            ));
        }
        if self.roles.contains_key(role.name) {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::DuplicateOutputRole,
                role.name,
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
        Action: action::Sealed,
    {
        self.binding_type = Some(TypeId::of::<Binding>());
        self.roles.insert(
            role.name.to_owned(),
            CandidateOutputBinding {
                posture: Action::POSTURE,
                entity_name: target.entity.clone(),
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
        Action: action::Sealed,
    {
        self.validate_binding(role, target, program)?;
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
        for (role, binding) in &self.roles {
            let expected = self
                .expected_roles
                .get(role)
                .expect("every admitted role came from the prepared contract");
            if expected.posture != binding.posture || expected.entity_name != binding.entity_name {
                return Err(denial(
                    WorthQueryApplicationAttemptDenialKind::OutputRoleEntityMismatch,
                    role,
                ));
            }
            let is_deleted = match binding.entity {
                EntityReference::Existing(entity_id) => effects.iter().any(|effect| {
                    matches!(effect, WorthQueryApplicationRealizedEffect::DeleteEntity { entity_id: deleted } if *deleted == entity_id)
                }),
                EntityReference::Created(_) => false,
            };
            let action_matches = match binding.posture {
                WorthQueryApplicationOutputPosture::Preserve => !is_deleted,
                WorthQueryApplicationOutputPosture::Create => {
                    matches!(
                        &binding.entity,
                        EntityReference::Created(created)
                            if effects.iter().any(|effect| created_reference_matches_effect(created, effect))
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

fn created_reference_matches_effect(
    created: &worth_relational::facade::transactions::CreatedEntityRef,
    effect: &WorthQueryApplicationRealizedEffect,
) -> bool {
    matches!(
        effect,
        WorthQueryApplicationRealizedEffect::CreateEntity { kind, key, .. }
            if created.partition_id == worth_relational::facade::identity::PartitionId::main()
                && created.kind_id == *kind
                && created.client_key == worth_relational::facade::symbols::ClientKey::raw(key)
    )
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
    if role.is_empty()
        || role.trim() != role
        || role.len() > 256
        || role.chars().any(char::is_control)
    {
        Err(denial(
            WorthQueryApplicationAttemptDenialKind::InvalidOutputRole,
            role,
        ))
    } else {
        Ok(())
    }
}

fn denial(
    kind: WorthQueryApplicationAttemptDenialKind,
    subject: impl Into<String>,
) -> WorthQueryApplicationAttemptDenial {
    WorthQueryApplicationAttemptDenial::new(kind, subject)
}
