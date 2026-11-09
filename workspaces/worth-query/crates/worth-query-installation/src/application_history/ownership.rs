//! Resolve every declared edge against the actual installed schema and model root.
use super::{
    WorthQueryDocumentHistoryInstallationDenial as Denial,
    WorthQueryDocumentHistoryInstallationDenialKind as Kind,
};
use worth_query_declaration::facade::{
    application_history::{history_ownership_reaches, WorthQueryPortableApplicationHistoryParts},
    application_schema::ApplicationSchemaMember,
};

pub(super) fn validate(
    parts: &WorthQueryPortableApplicationHistoryParts,
    members: &[ApplicationSchemaMember],
) -> Result<(), Denial> {
    let root = &parts.access.parts().scope_entity;
    for ownership in &parts.ownership {
        let declared = members.iter().any(|member| matches!(member,
            ApplicationSchemaMember::Relation { relation, from, to, .. }
                if relation == &ownership.relation && from == &ownership.from && to == &ownership.to));
        if !declared || !history_ownership_reaches(&parts.ownership, root, &ownership.from) {
            return Err(Denial::new(
                Kind::BindingMeaningChanged,
                &ownership.relation,
            ));
        }
    }
    Ok(())
}
