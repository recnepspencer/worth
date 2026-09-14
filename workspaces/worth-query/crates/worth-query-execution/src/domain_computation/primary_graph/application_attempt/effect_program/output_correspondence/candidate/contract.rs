use std::any::TypeId;
use std::collections::BTreeMap;

use worth_query_declaration::facade::application_operation::{
    ApplicationMutationBinding, ApplicationMutationOutputContract,
    ApplicationMutationOutputPostureSet, ApplicationMutationOutputRoleFamilyDescriptor,
};
use worth_query_installation::facade::ApplicationSchema;

use super::{denial, WorthQueryApplicationOutputCorrespondenceCandidate};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
    WorthQueryApplicationOutputPosture,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct ExpectedOutputBinding {
    pub(super) posture: WorthQueryApplicationOutputPosture,
    pub(super) entity_name: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct ExpectedOutputFamily {
    pub(super) prefix: String,
    pub(super) postures: ApplicationMutationOutputPostureSet,
    pub(super) entity_name: &'static str,
    pub(super) minimum: usize,
}

impl WorthQueryApplicationOutputCorrespondenceCandidate {
    pub(super) fn prepare_contract<Schema, Binding>(
        &self,
    ) -> Result<
        (
            TypeId,
            BTreeMap<String, ExpectedOutputBinding>,
            Vec<ExpectedOutputFamily>,
            usize,
        ),
        WorthQueryApplicationAttemptDenial,
    >
    where
        Schema: ApplicationSchema,
        Binding: ApplicationMutationBinding<Schema>,
    {
        let mut prepared = BTreeMap::new();
        let mut families = Vec::new();
        let mut retained_representation_bytes = 0_usize;
        for descriptor in <Binding::Output as ApplicationMutationOutputContract<Schema>>::ROLES {
            super::validate_role_name(descriptor.name())?;
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
        for family in <Binding::Output as ApplicationMutationOutputContract<Schema>>::ROLE_FAMILIES
        {
            validate_family(family, prepared.keys(), &families)?;
            retained_representation_bytes = retained_representation_bytes
                .checked_add(family.prefix().len())
                .ok_or_else(|| {
                    denial(
                        WorthQueryApplicationAttemptDenialKind::CandidateReservationExceeded,
                        family.prefix(),
                    )
                })?;
            families.push(ExpectedOutputFamily {
                prefix: family.prefix().to_owned(),
                postures: family.postures(),
                entity_name: family.entity(),
                minimum: family.minimum(),
            });
        }
        Ok((
            TypeId::of::<Binding>(),
            prepared,
            families,
            retained_representation_bytes,
        ))
    }
}

fn validate_family<'a>(
    family: &ApplicationMutationOutputRoleFamilyDescriptor,
    exact_roles: impl Iterator<Item = &'a String>,
    families: &[ExpectedOutputFamily],
) -> Result<(), WorthQueryApplicationAttemptDenial> {
    let prefix = family.prefix();
    if !valid_family_prefix(prefix)
        || family.postures().is_empty()
        || exact_roles.into_iter().any(|role| role.starts_with(prefix))
        || families.iter().any(|existing| {
            existing.prefix.starts_with(prefix) || prefix.starts_with(&existing.prefix)
        })
    {
        return Err(denial(
            WorthQueryApplicationAttemptDenialKind::InvalidOutputRole,
            prefix,
        ));
    }
    Ok(())
}

fn valid_family_prefix(prefix: &str) -> bool {
    prefix.ends_with('.')
        && prefix.len() > 1
        && super::validate_role_name(prefix).is_ok()
        && !prefix.starts_with('.')
        && !prefix.contains("..")
}
