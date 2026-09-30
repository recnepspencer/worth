mod authoring;
mod completion;
mod contract;

use std::any::TypeId;
use std::collections::BTreeMap;

use worth_query_declaration::facade::application_operation::ApplicationMutationOutputRoleCardinality;
use worth_relational::facade::identity::EntityId;
use worth_relational::facade::transactions::{CommitResult, EntityReference};

use contract::{ExpectedOutputBinding, ExpectedOutputFamily};

use super::{
    CommittedOutputBinding, OutputRoleUse, WorthQueryApplicationOutputCorrespondence,
    WorthQueryApplicationOutputPosture, WorthQueryApplicationOutputRoleNameDenial,
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
    contract_type: Option<TypeId>,
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

    /// Expect the roles and families `Contract` declares, as a binding with
    /// that output contract would.
    #[cfg(test)]
    pub(super) fn prepare_test_contract<Schema, Contract>(&mut self)
    where
        Schema: worth_query_installation::facade::ApplicationSchema,
        Contract: worth_query_declaration::facade::application_operation::ApplicationMutationOutputContract<Schema>,
    {
        let (roles, families, _) = self
            .prepare_contract::<Schema, Contract>()
            .expect("the test contract is well formed");
        self.contract_type = Some(TypeId::of::<Contract>());
        self.expected_roles.extend(roles);
        self.expected_families.extend(families);
    }

    fn validate_binding<Schema, Entity>(
        &self,
        role: &OutputRoleUse,
        target: &WorthQueryApplicationEffectEntity<Schema, Entity>,
        program: &std::sync::Arc<()>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        let posture = role.posture;
        let name = role.name.as_str();
        validate_role_name(name)?;
        if !std::sync::Arc::ptr_eq(program, &target.program) {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::ForeignEffectTarget,
                name,
            ));
        }
        validate_reference_posture(posture, &target.reference, name)?;
        if self
            .contract_type
            .is_some_and(|existing| existing != role.contract_type)
        {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::ForeignOutputRole,
                name,
            ));
        }
        let exact = self.expected_roles.get(name);
        let family = self
            .expected_families
            .iter()
            .find(|family| family_matches(name, &family.prefix));
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
                    name,
                ))
            }
        };
        if expected_entity != target.entity || expected_entity != role.entity_name {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::OutputRoleEntityMismatch,
                name,
            ));
        }
        if !posture_allowed {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::OutputRoleActionMismatch,
                name,
            ));
        }
        if cardinality != role.cardinality {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::OutputRoleCardinalityMismatch,
                name,
            ));
        }
        if self.roles.contains_key(name) {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::DuplicateOutputRole,
                name,
            ));
        }
        Ok(())
    }

    fn insert_binding<Schema, Entity>(
        &mut self,
        role: OutputRoleUse,
        target: &WorthQueryApplicationEffectEntity<Schema, Entity>,
    ) {
        self.contract_type = Some(role.contract_type);
        self.roles.insert(
            role.name,
            CandidateOutputBinding {
                posture: role.posture,
                entity_name: target.entity.clone(),
                entity_type: role.entity_type,
                entity: target.reference.clone(),
            },
        );
    }

    #[cfg(test)]
    pub(super) fn bind<Schema, Entity>(
        &mut self,
        role: OutputRoleUse,
        target: &WorthQueryApplicationEffectEntity<Schema, Entity>,
        program: &std::sync::Arc<()>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
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
            contract_type: self.contract_type,
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
    if WorthQueryApplicationOutputRoleNameDenial::validate(role).is_err() {
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
