use std::collections::BTreeSet;

use crate::application_schema::{
    ApplicationSchemaDeclarationDenial as Denial, ApplicationSchemaMember as Member,
};
use crate::portable_identity::WorthQueryPortableTypeIdentity;

pub(in crate::application_schema) fn validate_closure(members: &[Member]) -> Result<(), Denial> {
    let mut identities = members
        .iter()
        .filter_map(|member| match member {
            Member::Aspect { identity, .. } => Some(*identity),
            _ => None,
        })
        .collect::<BTreeSet<_>>();
    let mut loci = BTreeSet::new();
    for definition in members.iter().filter_map(|member| match member {
        Member::RelationAspect { definition } => Some(definition),
        _ => None,
    }) {
        definition
            .validate()
            .map_err(|_| Denial::InvalidApplicationRelationAspect)?;
        let aspect = definition.parts();
        if !loci.insert((&aspect.relation, &aspect.aspect)) || !identities.insert(aspect.identity) {
            return Err(Denial::DuplicateMember);
        }
        if !members.iter().any(|member| {
            matches!(member, Member::Relation { relation, from, to, .. }
            if relation == &aspect.relation && from == &aspect.from && to == &aspect.to)
        }) {
            return Err(Denial::MissingApplicationRelationAspectDependency);
        }
        for field in &aspect.fields {
            if !WorthQueryPortableTypeIdentity::from_untrusted(field.value_type.clone()).is_valid()
            {
                return Err(Denial::InvalidApplicationRelationAspect);
            }
            if let Some(unit) = &field.unit {
                if !members.iter().any(|member| matches!(member, Member::Unit { unit: installed } if installed == unit)) {
                    return Err(Denial::MissingUnit);
                }
            }
        }
    }
    Ok(())
}
