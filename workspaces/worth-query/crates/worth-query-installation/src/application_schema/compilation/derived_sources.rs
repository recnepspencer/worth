//! The closed framework count owner retains its genuine EntityIdentity input.
use worth_query_declaration::facade::application_schema::ApplicationSchemaMember as Member;

pub(super) fn validate(members: &[Member]) -> Result<(), String> {
    // Native now evaluates the complete declared law/source closure. The only
    // remaining availability limit here belongs to the closed count Input,
    // not to logical rule, decision, walk, orbit or component sources.
    for member in members {
        let Member::RuleReadableCollection { association } = member else {
            continue;
        };
        let from = members.iter().find_map(|member| match member {
            Member::DerivedRelation { definition }
                if definition.parts().relation == association.relation =>
            {
                Some(&definition.parts().from)
            }
            _ => None,
        });
        if !from.is_some_and(|member| member.entity().is_some()) {
            return Err(association.relation.clone());
        }
    }
    // Actual contribution seal still requires the framework's real closed
    // typed owner; equal descriptive association/custom Value grants nothing.
    Ok(())
}
