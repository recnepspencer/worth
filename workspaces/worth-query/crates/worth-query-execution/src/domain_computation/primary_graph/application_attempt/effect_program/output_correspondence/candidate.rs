mod authoring;
mod completion;
mod contract;

use std::any::TypeId;
use std::collections::BTreeMap;

#[cfg(test)]
use worth_query_declaration::facade::application_operation::ApplicationMutationOutputPostureSet;
use worth_query_declaration::facade::application_operation::ApplicationMutationOutputRoleCardinality;
use worth_relational::facade::identity::EntityId;
use worth_relational::facade::transactions::{CommitResult, EntityReference};

use contract::{ExpectedOutputBinding, ExpectedOutputFamily};

use super::{
    CommittedOutputBinding, WorthQueryApplicationFixedOutputRole,
    WorthQueryApplicationOutputAction, WorthQueryApplicationOutputCorrespondence,
    WorthQueryApplicationOutputPosture,
};
use crate::domain_computation::primary_graph::application_attempt::effect_program::WorthQueryApplicationEffectEntity;
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
    pub(super) fn prepare_test_role<Role>(&mut self, role: &Role, entity_name: &'static str)
    where
        Role: WorthQueryApplicationFixedOutputRole,
    {
        self.binding_type = Some(TypeId::of::<Role::Binding>());
        self.expected_roles.insert(
            role.name().to_owned(),
            ExpectedOutputBinding {
                posture: <Role::Action as WorthQueryApplicationOutputAction>::POSTURE,
                entity_name,
                cardinality: Role::cardinality(super::fixed_output_role::INTERNAL),
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

    fn validate_binding<Schema, Role>(
        &self,
        role: &Role,
        target: &WorthQueryApplicationEffectEntity<Schema, Role::Entity>,
        program: &std::sync::Arc<()>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial>
    where
        Role: WorthQueryApplicationFixedOutputRole,
    {
        let posture = <Role::Action as WorthQueryApplicationOutputAction>::POSTURE;
        validate_role_name(role.name())?;
        if !std::sync::Arc::ptr_eq(program, &target.program) {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::ForeignEffectTarget,
                role.name(),
            ));
        }
        validate_reference_posture(posture, &target.reference, role.name())?;
        let binding_type = TypeId::of::<Role::Binding>();
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
        let (posture_allowed, expected_entity, cardinality) = match (exact, family) {
            (Some(expected), None) => (
                expected.posture == posture,
                expected.entity_name,
                expected.cardinality,
            ),
            (None, Some(expected)) => (
                expected.postures.allows(posture),
                expected.entity_name,
                ApplicationMutationOutputRoleCardinality::ExactlyOne,
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
        if cardinality != Role::cardinality(super::fixed_output_role::INTERNAL) {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::OutputRoleCardinalityMismatch,
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

    fn insert_binding<Schema, Role>(
        &mut self,
        role: Role,
        target: &WorthQueryApplicationEffectEntity<Schema, Role::Entity>,
    ) where
        Role: WorthQueryApplicationFixedOutputRole,
    {
        self.binding_type = Some(TypeId::of::<Role::Binding>());
        self.roles.insert(
            role.name().to_owned(),
            CandidateOutputBinding {
                posture: <Role::Action as WorthQueryApplicationOutputAction>::POSTURE,
                entity_name: target.entity.clone(),
                entity_type: TypeId::of::<Role::Entity>(),
                entity: target.reference.clone(),
            },
        );
    }

    #[cfg(test)]
    pub(super) fn bind<Schema, Role>(
        &mut self,
        role: Role,
        target: &WorthQueryApplicationEffectEntity<Schema, Role::Entity>,
        program: &std::sync::Arc<()>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial>
    where
        Role: WorthQueryApplicationFixedOutputRole,
    {
        self.validate_binding(&role, target, program)?;
        self.insert_binding(role, target);
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
        let optional_roles = self
            .expected_roles
            .into_iter()
            .filter(|(_, expected)| expected.cardinality.admits_absence())
            .map(|(role, _)| role)
            .collect();
        WorthQueryApplicationOutputCorrespondence {
            binding_type: self.binding_type,
            optional_roles,
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
