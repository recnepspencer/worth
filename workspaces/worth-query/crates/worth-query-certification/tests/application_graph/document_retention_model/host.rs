//! Publishing a document-retention host through the real installation path.
//!
//! Every host in this court is built here and nowhere else: one installed
//! schema carrying both rule contracts, an explicit program roster, and two
//! seeded documents. Ordinary cases share one resource profile; the scheduled
//! history court explicitly installs larger history and exact-pin capacities.

use worth_query_host::facade::application_contribution::{
    WorthQueryApplicationContribution, WorthQueryApplicationContributionSetup,
};
use worth_query_host::facade::application_installation::{
    in_memory_rostered_program, WorthQueryApplicationProgramRoster,
    WorthQueryInMemoryApplicationDenial, WorthQueryInMemoryApplicationLimits,
    WorthQueryInMemoryApplicationProfile, WorthQueryProgramApplicationRuntime,
};
use worth_query_host::facade::declaration::application_program::{
    ApplicationProgramDefinition, ApplicationProgramOutputsShape, ValidatedApplicationProgram,
};
use worth_query_host::facade::declaration::application_schema::{
    ApplicationInvariantExecutionPoint, ApplicationInvariantMarkerIdentity,
};
use worth_query_host::facade::domain::WorthQueryInstalledApplicationSchema;
use worth_query_host::facade::{declaration, primary_graph, runtime};

#[path = "host/world_resources.rs"]
mod world_resources;
use world_resources::world_resources;

use super::assessment_output::{
    RetentionAssessmentBinding, RetentionAssessmentHandler, RetentionAssessmentProducer,
    RetentionAssessmentProvider,
};
use super::assessment_readiness::RetentionAssessmentReadiness;
use super::programs::{
    validated_changed_feature_program, validated_changed_operation_program,
    validated_first_program, validated_first_resource_program, validated_foreign_rule_program,
    validated_removed_assessment_supplier_program, validated_removed_operation_program,
    validated_second_program, validated_second_resource_program, ResourceRetentionProgramP0,
    RetentionProgramP0, RetentionProgramP1,
};
use super::retention_entry::{
    ReviewedSetRetentionBinding, ReviewedSetRetentionHandler, SetRetentionBinding,
    SetRetentionHandler, DOCUMENT_IDENTITY, RELATED_DOCUMENT_IDENTITY,
};
use super::rules::{resolve_first_rule, resolve_second_rule};
use super::schema::{
    Document, DocumentIdentityField, DocumentPrincipalBinding, DocumentRetentionContribution,
    DocumentRetentionField, DocumentRetentionSchema, DocumentRetentionV1, DocumentRetentionV2,
};
use super::workflow::{
    ReviewRequirementBinding, ReviewRequirementHandler, UnlinkReviewRequirementBinding,
    UnlinkReviewRequirementHandler, WorkflowGrantStatusBinding, WorkflowGrantStatusHandler,
};

#[path = "host/checkpoint_restore.rs"]
mod checkpoint_restore;
#[path = "host/trusted_time.rs"]
mod trusted_time;
#[path = "host/workflow_runtime.rs"]
mod workflow_runtime;
pub use checkpoint_restore::{restore, restore_on_first_program};
pub use trusted_time::{publish_on_first_program_with_trusted_time, CertificationTrustedTime};
pub use workflow_runtime::DocumentWorkflowRuntime;

/// The retention every host seeds. It satisfies both installed rules, so the
/// same bootstrap is lawful whichever program the host starts on.
pub const SEED_RETENTION: u64 = 7;

#[cfg(test)]
#[path = "host/tests.rs"]
mod tests;

pub type DocumentRetentionRuntime<Initial> =
    WorthQueryProgramApplicationRuntime<DocumentRetentionSchema, Initial>;

impl WorthQueryApplicationContribution<DocumentRetentionSchema> for DocumentRetentionContribution {
    type Configuration = ();

    fn contracts(
        contracts: &mut worth_query_host::facade::application_contribution::WorthQueryApplicationContributionContracts<DocumentRetentionSchema>,
    ) -> Result<(), primary_graph::WorthQueryPrimaryGraphInstallationDenial> {
        contracts.producer::<RetentionAssessmentProducer>()?;
        contracts.conditional::<RetentionAssessmentReadiness>()?;
        Ok(())
    }

    fn configure(
        (): Self::Configuration,
        setup: &mut WorthQueryApplicationContributionSetup<'_, DocumentRetentionSchema>,
    ) -> Result<(), primary_graph::WorthQueryPrimaryGraphInstallationDenial> {
        setup.invariant(
            DocumentRetentionV1::reference(),
            ApplicationInvariantExecutionPoint::CommitBoundary,
            resolve_first_rule,
        )?;
        setup.invariant(
            DocumentRetentionV2::reference(),
            ApplicationInvariantExecutionPoint::CommitBoundary,
            resolve_second_rule,
        )?;
        setup
            .handler::<SetRetentionBinding, _>(SetRetentionHandler)
            .and_then(|()| {
                setup.handler::<ReviewedSetRetentionBinding, _>(ReviewedSetRetentionHandler)
            })
            .and_then(|()| setup.handler::<ReviewRequirementBinding, _>(ReviewRequirementHandler))
            .and_then(|()| {
                setup.handler::<UnlinkReviewRequirementBinding, _>(UnlinkReviewRequirementHandler)
            })
            .and_then(|()| {
                setup.handler::<RetentionAssessmentBinding, _>(RetentionAssessmentHandler)
            })
            .and_then(|()| {
                setup.producer::<RetentionAssessmentProducer>(RetentionAssessmentProvider)
            })
            .and_then(|()| setup.conditional::<RetentionAssessmentReadiness>(()))
            .and_then(|()| {
                setup.handler::<WorkflowGrantStatusBinding, _>(WorkflowGrantStatusHandler)
            })
    }
}

/// Publishes a host whose first occurrence runs P0, with P1 rostered beside it.
pub fn publish_on_first_program() -> DocumentRetentionRuntime<RetentionProgramP0> {
    publish_on_first_program_with_limits(host_limits())
}

/// Scheduled 10k publication lane: 200k candidate items, 128 MiB candidate
/// bytes, 20M candidate work, 200k operation width, and WorkflowScale's
/// finite Relational publication and integrity ceilings. Ordinary actions retain smaller
/// handler-level candidate requirements.
pub fn publish_on_first_program_for_workflow_scale() -> DocumentRetentionRuntime<RetentionProgramP0>
{
    publish_on_first_program_with_limits(
        WorthQueryInMemoryApplicationLimits::new(
            world_resources(1_024, 256 * 1024 * 1024, 2_048),
            runtime::WorthQueryApplicationCandidateResourceProfile::bounded(
                200_000,
                128 * 1024 * 1024,
                20_000_000,
            )
            .expect("finite workflow-scale candidate resources")
            .with_maximum_operation_width(200_000)
            .expect("finite workflow-scale publication width"),
            runtime::WorthQueryApplicationQueryResourceProfile::bounded(5_120, 2_048, 200_000, 128)
                .expect("finite workflow-scale query resources"),
            primary_graph::SignalConditionalEvaluationBudget::development(),
        )
        .with_profile(WorthQueryInMemoryApplicationProfile::WorkflowScale),
    )
}

fn publish_on_first_program_with_limits(
    limits: WorthQueryInMemoryApplicationLimits,
) -> DocumentRetentionRuntime<RetentionProgramP0> {
    publish_with_limits(
        validated_first_program(),
        WorthQueryApplicationProgramRoster::new()
            .support(validated_second_program())
            .support(validated_changed_feature_program())
            .support(validated_changed_operation_program())
            .support(validated_removed_assessment_supplier_program())
            .support(validated_removed_operation_program()),
        limits,
    )
    .expect("the P0-initial document-retention host must install")
}

pub fn publish_workflow_on_first_program() -> DocumentWorkflowRuntime {
    super::workflow::retain_workflow(publish_on_first_program())
}

/// Publishes a host whose first occurrence runs P1, with P0 rostered beside it.
pub fn publish_on_second_program() -> DocumentRetentionRuntime<RetentionProgramP1> {
    publish(
        validated_second_program(),
        WorthQueryApplicationProgramRoster::new()
            .support(validated_first_program())
            .support(validated_changed_feature_program())
            .support(validated_changed_operation_program())
            .support(validated_removed_operation_program()),
    )
    .expect("the P1-initial document-retention host must install")
}

pub fn publish_on_first_resource_program() -> DocumentRetentionRuntime<ResourceRetentionProgramP0> {
    publish(
        validated_first_resource_program(),
        WorthQueryApplicationProgramRoster::new()
            .support(validated_second_resource_program())
            .support(validated_second_program()),
    )
    .expect("the resource-custody host must install")
}

/// Attempts a host that rosters P0 alone against the two-rule catalog.
pub fn publish_first_program_alone(
) -> Result<DocumentRetentionRuntime<RetentionProgramP0>, WorthQueryInMemoryApplicationDenial> {
    publish(
        validated_first_program(),
        WorthQueryApplicationProgramRoster::new(),
    )
}

/// Attempts a host that rosters a program declaring an uninstalled rule.
pub fn publish_with_foreign_rule_rostered(
) -> Result<DocumentRetentionRuntime<RetentionProgramP0>, WorthQueryInMemoryApplicationDenial> {
    publish(
        validated_first_program(),
        WorthQueryApplicationProgramRoster::new()
            .support(validated_second_program())
            .support(validated_changed_feature_program())
            .support(validated_changed_operation_program())
            .support(validated_removed_operation_program())
            .support(validated_foreign_rule_program()),
    )
}

pub fn publish<Initial>(
    initial: ValidatedApplicationProgram<DocumentRetentionSchema, Initial>,
    roster: WorthQueryApplicationProgramRoster<'_, DocumentRetentionSchema>,
) -> Result<DocumentRetentionRuntime<Initial>, WorthQueryInMemoryApplicationDenial>
where
    Initial: ApplicationProgramDefinition<
            DocumentRetentionSchema,
            Contributions = (DocumentRetentionContribution,),
        > + 'static,
    Initial::Outputs:
        ApplicationProgramOutputsShape<DocumentRetentionSchema>
            + worth_query_host::facade::application_installation::WorthQueryApplicationProgramRoots<
                DocumentRetentionSchema,
            >,
{
    publish_with_limits(initial, roster, host_limits())
}

fn publish_with_limits<Initial>(
    initial: ValidatedApplicationProgram<DocumentRetentionSchema, Initial>,
    roster: WorthQueryApplicationProgramRoster<'_, DocumentRetentionSchema>,
    limits: WorthQueryInMemoryApplicationLimits,
) -> Result<DocumentRetentionRuntime<Initial>, WorthQueryInMemoryApplicationDenial>
where
    Initial: ApplicationProgramDefinition<
            DocumentRetentionSchema,
            Contributions = (DocumentRetentionContribution,),
        > + 'static,
    Initial::Outputs:
        ApplicationProgramOutputsShape<DocumentRetentionSchema>
            + worth_query_host::facade::application_installation::WorthQueryApplicationProgramRoots<
                DocumentRetentionSchema,
            >,
{
    in_memory_rostered_program(
        initial,
        roster,
        DocumentRetentionSchema::declaration().expect("the document-retention schema is valid"),
        ((),),
        limits,
        seed_host,
    )
}

fn seed_host(
    graph: &mut primary_graph::WorthQueryPrimaryGraphBootstrap<DocumentRetentionSchema>,
    installed: &WorthQueryInstalledApplicationSchema<DocumentRetentionSchema>,
) -> Result<(), primary_graph::WorthQueryPrimaryGraphInstallationDenial> {
    let principal_binding = installed
        .principal_binding(DocumentPrincipalBinding::reference())
        .expect("the document principal binding must install");
    seed_operator(graph, &principal_binding);
    seed_document(graph);
    super::workflow::seed_authoring(graph);
    Ok(())
}

fn seed_operator(
    graph: &mut primary_graph::WorthQueryPrimaryGraphBootstrap<DocumentRetentionSchema>,
    principal_binding: &worth_query_host::facade::domain::WorthQueryInstalledPrincipalBinding<
        DocumentRetentionSchema,
        DocumentPrincipalBinding,
        super::schema::ExternalMapping,
        super::schema::Principal,
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
            super::operator_identity::operator_identity(),
            declaration::authentication::WorthQueryPrincipalMappingStatus::Enabled,
        )
        .expect("the operator must seed");
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

fn host_limits() -> WorthQueryInMemoryApplicationLimits {
    // Installation admits the binding's maximum publication shape even though
    // the ordinary Document handler requests its narrow candidate at execution.
    WorthQueryInMemoryApplicationLimits::new(
        world_resources(256, 4 * 1024 * 1024, 256),
        runtime::WorthQueryApplicationCandidateResourceProfile::bounded(
            200_000,
            128 * 1024 * 1024,
            20_000_000,
        )
        .expect("valid candidate limits"),
        runtime::WorthQueryApplicationQueryResourceProfile::bounded(5_120, 2_048, usize::MAX, 128)
            .expect("valid query limits"),
        primary_graph::SignalConditionalEvaluationBudget::development(),
    )
}
