//! Genuine public bootstrap and program installation for the neutral example.
use crate::document_retention_model::{
    host::{host_limits, SEED_RETENTION},
    programs::validated_second_program,
    retention_entry::{DOCUMENT_IDENTITY, RELATED_DOCUMENT_IDENTITY},
    schema::*,
};
use worth_query_host::facade::application_contribution::WorthQueryBootstrapAdvancementPhase;
use worth_query_host::facade::application_installation::{
    in_memory_rostered_program, WorthQueryApplicationProgramRoster,
};
use worth_query_host::facade::domain::WorthQueryInstalledApplicationSchema;
use worth_query_host::facade::{declaration, primary_graph};

pub fn publish() -> crate::document_retention_model::MixedApplication {
    let application = in_memory_rostered_program(
        crate::program::validated_program(),
        WorthQueryApplicationProgramRoster::new().support(validated_second_program()),
        DocumentRetentionSchema::declaration().expect("the neutral schema validates"),
        ((), ()),
        host_limits(),
        seed_host,
    )
    .expect("the neutral mixed program installs through the real bootstrap");
    crate::document_retention_model::retain_mixed_workflow(application)
}

fn seed_host(
    _phase: &WorthQueryBootstrapAdvancementPhase<'_>,
    graph: &mut primary_graph::WorthQueryPrimaryGraphBootstrap<DocumentRetentionSchema>,
    installed: &WorthQueryInstalledApplicationSchema<DocumentRetentionSchema>,
) -> Result<(), primary_graph::WorthQueryPrimaryGraphInstallationDenial> {
    let principal_binding = installed
        .principal_binding(DocumentPrincipalBinding::reference())
        .expect("the document principal binding must install");
    seed_operator(graph, &principal_binding);
    seed_document(graph);
    crate::document_retention_model::workflow::seed_authoring(graph);
    crate::document_retention_model::workflow::governed_read::seed(graph);
    crate::document_retention_model::program_adoption::seed(graph);
    Ok(())
}

fn seed_operator(
    graph: &mut primary_graph::WorthQueryPrimaryGraphBootstrap<DocumentRetentionSchema>,
    principal_binding: &worth_query_host::facade::domain::WorthQueryInstalledPrincipalBinding<
        DocumentRetentionSchema,
        DocumentPrincipalBinding,
        crate::document_retention_model::schema::ExternalMapping,
        crate::document_retention_model::schema::Principal,
        u64,
        declaration::application_schema::U64ApplicationValueBinding,
    >,
) {
    graph
        .bind_principal(
            principal_binding,
            primary_graph::WorthQueryApplicationPrincipalKey::new("document-retention-operator")
                .expect("the principal key is valid"),
            1_u64,
            crate::document_retention_model::operator_identity::operator_identity(),
            declaration::authentication::WorthQueryPrincipalMappingStatus::Enabled,
        )
        .expect("the operator must seed");
    for (key, identity, external) in [
        (
            "document-retention-reviewer",
            2_u64,
            crate::document_retention_model::operator_identity::reviewer_identity(),
        ),
        (
            "document-retention-ungranted-reviewer",
            3_u64,
            crate::document_retention_model::operator_identity::ungranted_reviewer_identity(),
        ),
    ] {
        graph
            .bind_principal(
                principal_binding,
                primary_graph::WorthQueryApplicationPrincipalKey::new(key).unwrap(),
                identity,
                external,
                declaration::authentication::WorthQueryPrincipalMappingStatus::Enabled,
            )
            .expect("review principals seed through the native principal binding");
    }
}

fn seed_document(
    graph: &mut primary_graph::WorthQueryPrimaryGraphBootstrap<DocumentRetentionSchema>,
) {
    graph
        .bind_entity(
            primary_graph::WorthQueryApplicationEntitySeed::new(
                Document::reference(),
                primary_graph::WorthQueryApplicationEntityKey::new("document-row-1")
                    .expect("the row key is valid"),
            )
            .field(
                DocumentIdentityField::reference(),
                DOCUMENT_IDENTITY.to_owned(),
            )
            .field(DocumentRetentionField::reference(), SEED_RETENTION),
        )
        .expect("the document must seed");
    graph
        .bind_entity(
            primary_graph::WorthQueryApplicationEntitySeed::new(
                Document::reference(),
                primary_graph::WorthQueryApplicationEntityKey::new("document-row-2")
                    .expect("the related row key is valid"),
            )
            .field(
                DocumentIdentityField::reference(),
                RELATED_DOCUMENT_IDENTITY.to_owned(),
            )
            .field(DocumentRetentionField::reference(), SEED_RETENTION),
        )
        .expect("the related document must seed");
}
