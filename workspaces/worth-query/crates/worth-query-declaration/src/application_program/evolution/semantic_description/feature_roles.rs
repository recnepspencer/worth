//! Canonical required-role facts share the manifest's exact descriptive fields.
use super::{
    fact, framed_fields, framed_parts, ApplicationFeatureDeclaration, ApplicationSemanticFact,
    ApplicationSemanticFamily,
};
pub(super) fn append_required_roles(
    facts: &mut Vec<ApplicationSemanticFact>,
    feature: &ApplicationFeatureDeclaration,
) {
    for role in feature.required_roles() {
        facts.push(fact(
            ApplicationSemanticFamily::Outputs,
            framed_parts(&[
                feature.composition_instance(),
                feature.identity(),
                "required-role",
                role.kind().canonical_token(),
                role.producer(),
            ]),
            framed_fields(role.canonical_fields()),
        ));
    }
}
