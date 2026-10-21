//! Installs the real literal-body fixture and retains its projection owner.
use super::*;

pub(super) fn install<const DESTINATION: u64>() -> WorthQueryProgramApplicationRuntime<
    DocumentRetentionSchema<DESTINATION>,
    program::BodyProgram<DESTINATION>,
> {
    install_with_projection::<DESTINATION>().0
}

pub(super) fn install_with_projection<const DESTINATION: u64>() -> (
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
    let app = in_memory_program(
        program::validated::<DESTINATION>(&schema),
        schema,
        (program::Installation::Native,),
        ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap(),
        host_limits(),
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
                    .field(genesis::declaration::TenantIdentity::reference(), 1),
                )?;
                return Ok(());
            }
            let document = WorthQueryApplicationEntitySeed::new(
                Document::reference(),
                WorthQueryApplicationEntityKey::new("body-document").unwrap(),
            )
            .field(
                DocumentIdentityField::reference(),
                DOCUMENT_IDENTITY.to_owned(),
            )
            .field(DocumentRetentionField::reference(), 7)
            .field(DocumentAuditSourceField::reference(), 7);
            let document = if matches!(DESTINATION, 113 | 116) {
                document.field(
                    DocumentDraftNoteField::reference(),
                    DOCUMENT_IDENTITY.to_owned(),
                )
            } else if matches!(DESTINATION, 25 | 26 | 29) {
                document.field(DocumentDraftNoteField::reference(), "active".to_owned())
            } else if DESTINATION == 28 {
                document.field(DocumentDraftNoteField::reference(), String::new())
            } else {
                document
            };
            graph.bind_entity(document)?;
            output::related::seed_members::<DESTINATION>(graph)?;
            retirement::seed::seed::<DESTINATION>(graph)?;
            Ok(())
        },
    )
    .unwrap();
    (app, projection.unwrap())
}
