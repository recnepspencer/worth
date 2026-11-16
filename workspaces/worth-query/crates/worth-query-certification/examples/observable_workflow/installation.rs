//! Genuine public bootstrap and program installation for the neutral example.
use crate::document_retention_model::{
    host::{host_limits, SEED_RETENTION},
    programs::validated_second_program,
    retention_entry::{DOCUMENT_IDENTITY, RELATED_DOCUMENT_IDENTITY},
    schema::*,
};
use worth_query_host::facade::application_contribution::WorthQueryBootstrapAdvancementPhase;
use worth_query_host::facade::application_installation::ApplicationBuildIdentity;
use worth_query_host::facade::application_installation::WorthQueryApplicationProgramRoster;
use worth_query_host::facade::domain::WorthQueryInstalledApplicationSchema;
use worth_query_host::facade::{declaration, primary_graph};
use worth_query_host::facade::{
    WorthQueryApplicationHome, WorthQueryApplicationHomeLocation, WorthQueryApplicationHomeRequest,
};

pub type Home = WorthQueryApplicationHome<DocumentRetentionSchema, crate::program::MixedProgram>;

pub fn open(
    directory: &std::path::Path,
    configuration: crate::resource_monitor::Configuration,
) -> crate::document_retention_model::MixedApplication {
    let funding = funding();
    let home = worth_query_host::facade::application_contribution::with_bootstrap_advancement_from_request(
        worth_execution::ExecutionRequest::leased(&funding),
        |phase| WorthQueryApplicationHomeRequest::new(
            WorthQueryApplicationHomeLocation::Directory(directory.to_owned()),
            crate::program::validated_program(), roster(),
            DocumentRetentionSchema::declaration().expect("the neutral schema validates"),
            ((), (), configuration),
            ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap(),
            host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()),
            seed_host,
        ).open_in_advancement(&phase),
    ).expect("the original bootstrap request opens")
        .unwrap_or_else(|cause| panic!("neutral Directory Home: {cause:?}"));
    crate::document_retention_model::retain_mixed_workflow(home)
}

pub fn close(application: crate::document_retention_model::MixedApplication) {
    let funding = funding();
    let crate::document_retention_model::MixedApplication {
        workflow,
        authentication,
    } = application;
    drop(authentication);
    workflow
        .close(&worth_execution::ExecutionRequest::leased(&funding))
        .unwrap_or_else(|cause| panic!("neutral Home close retains its refusal: {cause:?}"))
        .reclaim();
}

fn funding() -> worth_execution::ExecutionResourceLease<'static> {
    use worth_execution::{CancellationToken, ExecutionResourceLease, LeaseRequest};
    use worth_foundational::{
        DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
    };
    ExecutionResourceLease::serial(LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(std::num::NonZeroUsize::MIN, Some(256 << 20), None),
        ),
        cancellation: CancellationToken::new(),
        deadline: None,
    })
    .expect("bounded Home lifecycle request")
}

fn roster() -> WorthQueryApplicationProgramRoster<'static, DocumentRetentionSchema> {
    // The reused contribution installs this binding; its original owner must
    // be rostered exactly as in the document model's normal host.
    WorthQueryApplicationProgramRoster::new()
        .support(validated_second_program())
        .support(crate::program::validated_second_program())
        .support(crate::document_retention_model::actor_generation::validated_actor_program())
        .support(
            crate::document_retention_model::model_lifecycle::validated_model_lifecycle_program(),
        )
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
                DocumentIdentityField::at("DocumentIdentityField"),
                DOCUMENT_IDENTITY.to_owned(),
            )
            .field(
                DocumentRetentionField::at("DocumentRetentionField"),
                SEED_RETENTION,
            ),
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
                DocumentIdentityField::at("DocumentIdentityField"),
                RELATED_DOCUMENT_IDENTITY.to_owned(),
            )
            .field(
                DocumentRetentionField::at("DocumentRetentionField"),
                SEED_RETENTION,
            ),
        )
        .expect("the related document must seed");
}
