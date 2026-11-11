//! Installs the real literal-body fixture and retains its projection owner.
pub use super::*;
#[path = "installation/admission_time.rs"]
mod admission_time;

pub fn install<const DESTINATION: u64>() -> WorthQueryProgramApplicationRuntime<
    DocumentRetentionSchema<DESTINATION>,
    program::BodyProgram<DESTINATION>,
> {
    install_with_projection::<DESTINATION>().0
}

pub fn install_with_projection<const DESTINATION: u64>() -> (
    WorthQueryProgramApplicationRuntime<
        DocumentRetentionSchema<DESTINATION>,
        program::BodyProgram<DESTINATION>,
    >,
    worth_query_host::facade::primary_graph::WorthQueryApplicationInvariantProjectionAuthority<
        DocumentRetentionSchema<DESTINATION>,
    >,
) {
    install_with_projection_on_clock::<DESTINATION>(
        worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system(),
    )
}

/// Installs this fixture with the same observer retained by admission and requests.
pub fn install_with_projection_on_clock<const DESTINATION: u64>(
    clock: worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock,
) -> (
    WorthQueryProgramApplicationRuntime<
        DocumentRetentionSchema<DESTINATION>,
        program::BodyProgram<DESTINATION>,
    >,
    worth_query_host::facade::primary_graph::WorthQueryApplicationInvariantProjectionAuthority<
        DocumentRetentionSchema<DESTINATION>,
    >,
) {
    let schema = program::schema::<DESTINATION>();
    let mut projection = None;
    let app = worth_query_host::facade::application_installation::in_memory_program_with_authorization_time_source(
        program::validated::<DESTINATION>(&schema),
        schema,
        (program::Installation::Native,),
        ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap(),
        host_limits(clock.clone()),
        |_phase, graph, installed| {
            projection = Some(graph.retain_invariant_projection_authority());
            graph.bind_principal(
                &installed
                    .principal_binding(DocumentPrincipalBinding::reference())
                    .unwrap(),
                WorthQueryApplicationPrincipalKey::new("body-operator").unwrap(),
                1,
                operator_identity(),
                WorthQueryPrincipalMappingStatus::Enabled,
            )?;
            if matches!(DESTINATION, 30 | 31) {
                graph.bind_entity(
                    WorthQueryApplicationEntitySeed::new(
                        genesis::declaration::Tenant::reference(),
                        WorthQueryApplicationEntityKey::new("static-tenant").unwrap(),
                    )
                    .field(genesis::declaration::TenantIdentity::at("TenantIdentity"), 1),
                )?;
                return Ok(());
            }
            let document = WorthQueryApplicationEntitySeed::new(
                Document::reference(),
                WorthQueryApplicationEntityKey::new("body-document").unwrap(),
            )
            .field(
                DocumentIdentityField::at("DocumentIdentityField"),
                DOCUMENT_IDENTITY.to_owned(),
            )
            .field(DocumentRetentionField::at("DocumentRetentionField"), 7)
            .field(DocumentAuditSourceField::at("DocumentAuditSourceField"), 7);
            let document = if matches!(DESTINATION, 113 | 116) {
                document.field(
                    DocumentDraftNoteField::at("DocumentDraftNoteField"),
                    DOCUMENT_IDENTITY.to_owned().into(),
                )
            } else if matches!(DESTINATION, 25 | 26 | 29) {
                document.field(DocumentDraftNoteField::at("DocumentDraftNoteField"), "active".to_owned().into())
            } else if DESTINATION == 28 {
                document.field(DocumentDraftNoteField::at("DocumentDraftNoteField"), String::new().into())
            } else {
                document
            };
            graph.bind_entity(document)?;
            output::related::seed_members::<DESTINATION>(graph)?;
            retirement::seed::seed::<DESTINATION>(graph)?;
            Ok(())
        },
        admission_time::FixtureAdmissionTime(clock),
    )
    .unwrap();
    (app, projection.unwrap())
}
