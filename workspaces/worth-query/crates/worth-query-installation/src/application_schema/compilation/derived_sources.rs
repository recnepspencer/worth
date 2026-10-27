//! Admit only graph laws actually supported by the Native derived source owner.
use worth_query_declaration::facade::application_schema::{
    application_derived_relation_physical_sources, ApplicationRelationCrossContextPolicy,
    ApplicationRelationGraphLaws, ApplicationSchemaMember as Member,
};

pub(super) fn validate(members: &[Member]) -> Result<(), String> {
    for member in members {
        let Member::DerivedRelation { definition } = member else {
            continue;
        };
        let parts = definition.parts();
        if parts.laws != ApplicationRelationGraphLaws::unbounded() {
            return Err(parts.relation.clone());
        }
        // Composition and reversal preserve same-context endpoints exactly
        // when every actual physical leg enforces that contract. Other laws
        // need their own genuine derived enforcement before admission.
        let sources = application_derived_relation_physical_sources(members, &parts.relation)
            .map_err(|_| parts.relation.clone())?;
        for source in sources {
            if !members.iter().any(|member|matches!(member,
                Member::Relation { relation,from,to,integrity,.. }
                    if relation==&source.relation && from==&source.from && to==&source.to
                        && integrity.endpoints.cross_context_policy==ApplicationRelationCrossContextPolicy::Forbid)) {
                return Err(parts.relation.clone());
            }
        }
    }
    // Collection meaning is still only descriptive here. Contribution seal
    // separately requires the actual closed typed collection owner; a custom
    // computation or an equal portable association cannot supply it.
    Ok(())
}
