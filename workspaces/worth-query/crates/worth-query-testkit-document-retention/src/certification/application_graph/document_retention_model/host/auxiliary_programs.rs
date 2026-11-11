//! Fresh and restored hosts install the same isolated lifecycle-handler coverage.
use super::{DocumentRetentionSchema, WorthQueryApplicationProgramRoster};
use worth_query_host::facade::declaration::application_program::{
    ApplicationProgramIdentity, ApplicationProgramRevision,
};

/// Include the same isolated lifecycle coverage as ordinary host publication.
pub fn with_lifecycle_programs<'host>(
    roster: WorthQueryApplicationProgramRoster<'host, DocumentRetentionSchema>,
    initial_identity: &ApplicationProgramIdentity,
    initial_revision: &ApplicationProgramRevision,
) -> WorthQueryApplicationProgramRoster<'host, DocumentRetentionSchema> {
    let model_lifecycle = super::super::model_lifecycle::validated_model_lifecycle_program();
    let roster = roster.support(super::super::actor_generation::validated_actor_program());
    // The initial program is already installed. Avoid rostering that exact
    // validated meaning twice without changing either initial program's meaning.
    if initial_identity == model_lifecycle.identity()
        && initial_revision == model_lifecycle.revision()
    {
        roster
    } else {
        roster.support(model_lifecycle)
    }
}
