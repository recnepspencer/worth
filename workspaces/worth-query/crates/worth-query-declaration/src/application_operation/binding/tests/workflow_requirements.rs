use super::*;
use crate::application_operation::ApplicationWorkflowExecutionRequirements;
use crate::application_program::*;
use crate::portable_identity::{WorthQueryPortableType, WorthQueryPortableTypeIdentity};

struct ReviewCapability;
impl WorthQueryPortableType for ReviewCapability {
    const PORTABLE_TYPE_NAME: &'static str = "test.workflow-review.v1";
    const PORTABLE_TYPE_IDENTITY: WorthQueryPortableTypeIdentity =
        WorthQueryPortableTypeIdentity::declared("test.workflow-review.v1");
}
impl crate::application_capability::ApplicationCapabilityMarkerIdentity for ReviewCapability {
    type Schema = MutationSchema;
    const IDENTIFIER: &'static str = "test.workflow-review";
}

macro_rules! guarded_binding {
    ($binding:ident $(, $requirements:expr)?) => {
        crate::worth_query_mutation_binding!(@binding
            $binding for RenameInput, schema MutationSchema,
            identity "test.guarded-rename.v1", input RenameInputBinding, operation RenameOperation,
            result RenameResultBinding,
            idempotency u64, identity "test.guarded-rename-key.v1",
            decision RenameDecision, denial RenameDenialBinding, handler identity "test.guarded-rename-handler.v1",
            program , workflow_authority required,
            workflow_requirements $($requirements)?,
            outputs RenameOutputs,
            principal PrincipalBinding, mapping ExternalMapping, principal_entity Account,
            principal_identity u64, identity_binding crate::application_schema::U64ApplicationValueBinding,
            scope_binding crate::application_operation::ApplicationMutationFieldScope<
                MutationSchema, Account, AccountIdentity, AccountIdField, u64,
                crate::application_schema::ReadOnly, crate::application_schema::NoApplicationUnit
            >,
            scope_field_type Account, AccountIdentity, AccountIdField, u64,
                crate::application_schema::ReadOnly, crate::application_schema::NoApplicationUnit,
            scope_field AccountIdField::reference(),
            candidates creates 0, deletes 0, links 0, unlinks 0, writes 1, emits 1,
            resources retained_representation_bytes 128
        );
    };
}

guarded_binding!(
    OccurrenceBinding,
    ApplicationWorkflowExecutionRequirements::occurrence_only()
);
guarded_binding!(
    ApprovalBinding,
    ApplicationWorkflowExecutionRequirements::requiring_approval::<ReviewCapability>()
);

struct Workflow;
struct EvidenceFeature;
impl ApplicationFeature<MutationSchema> for EvidenceFeature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "test.evidence-feature.v1";
}

struct EvidenceProgram<Binding>(std::marker::PhantomData<Binding>);
impl<Binding: ApplicationMutationBinding<MutationSchema>>
    ApplicationProgramDefinition<MutationSchema> for EvidenceProgram<Binding>
{
    type Contributions = ();
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("test.evidence-program.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![
            ApplicationFeatureSpec::root::<MutationSchema, EvidenceFeature>()
                .mutation::<Binding>()
                .finish(),
        ]
    }
}

#[test]
fn evidence_floor_changes_canonical_program_revision_and_operation_comparison() {
    fn meaning<Binding: ApplicationMutationBinding<MutationSchema>>(
    ) -> (ApplicationProgramRevision, ApplicationSemanticDescription) {
        let program =
            ApplicationProgramAuthoring::<MutationSchema, EvidenceProgram<Binding>>::begin()
                .validated_program()
                .unwrap();
        (
            program.revision().clone(),
            program.semantic_description().clone(),
        )
    }
    let ordinary = meaning::<RenameBinding>();
    let empty = meaning::<OccurrenceBinding>();
    let reviewed = meaning::<ApprovalBinding>();
    assert_ne!(ordinary.0, empty.0);
    assert_ne!(empty.0, reviewed.0);
    assert_ne!(ordinary.0, reviewed.0);
    for (source, target) in [(&ordinary.1, &empty.1), (&empty.1, &reviewed.1)] {
        let diff = ApplicationSemanticDiff::compare(source, target, 1024).unwrap();
        assert!(diff.changes().iter().any(|change| change.family()
            == ApplicationSemanticFamily::Operations
            && change.kind() == ApplicationSemanticChangeKind::Changed));
    }
}

impl ApplicationWorkflowSpec for Workflow {
    fn observation_policy() -> crate::application_program::ApplicationWorkflowObservationPolicy {
        crate::application_program::ApplicationWorkflowObservationPolicy::disabled()
    }
    fn actor_policy() -> crate::application_program::ApplicationWorkflowActorPolicy {
        crate::application_program::ApplicationWorkflowActorPolicy::not_applicable()
    }

    type Schema = MutationSchema;
    const IDENTITY: ApplicationWorkflowSpecIdentity =
        ApplicationWorkflowSpecIdentity::new("test.workflow.v1");
}

fn authored<Binding: ApplicationMutationBinding<MutationSchema>>(
) -> AuthoredWorkflowDefinition<Workflow> {
    let limits = ApplicationWorkflowDefinitionLimits::new(
        8,
        8,
        8,
        ApplicationWorkflowComponentLimits::new(1, 1, 1, 1, 1).unwrap(),
        8192,
    )
    .unwrap();
    let mut definition =
        ApplicationWorkflowDefinitionBuilder::<Workflow>::new("information", limits).unwrap();
    let propose = definition
        .operation_binding::<RenameBinding>("propose")
        .unwrap();
    let perform = definition.operation_binding::<Binding>("perform").unwrap();
    let done = definition.terminal("done").unwrap();
    definition
        .start(&propose)
        .control(
            &propose,
            ApplicationWorkflowControlOutcome::Completed,
            &perform,
        )
        .control(
            &perform,
            ApplicationWorkflowControlOutcome::Completed,
            &done,
        )
        .operation_input(&propose, &perform);
    definition.finish().unwrap()
}

#[test]
fn workflow_actor_contract_explicit_occurrence_allows_binding_without_approval() {
    assert!(authored::<OccurrenceBinding>().validate().is_ok());
    for denial in [authored::<ApprovalBinding>().validate().err()] {
        assert_eq!(
            denial.unwrap().kind(),
            ApplicationWorkflowValidationDenialKind::MissingWorkflowAuthority
        );
    }
}

#[test]
fn binding_description_distinguishes_inapplicable_occurrence_and_approval_evidence() {
    let ordinary = RenameBinding::descriptor();
    let empty = OccurrenceBinding::descriptor();
    let reviewed = ApprovalBinding::descriptor();
    assert_eq!(
        ordinary.workflow_execution_requirements(),
        &ApplicationWorkflowExecutionRequirements::not_applicable()
    );
    assert_ne!(ordinary.description(), empty.description());
    assert_ne!(empty.description(), reviewed.description());
}

#[test]
fn requirement_encoding_is_canonical_and_rejects_unknown_or_truncated_records() {
    for requirements in [
        ApplicationWorkflowExecutionRequirements::occurrence_only(),
        ApplicationWorkflowExecutionRequirements::requiring_approval::<ReviewCapability>(),
    ] {
        let encoded = requirements.persistence_identity();
        assert_eq!(
            ApplicationWorkflowExecutionRequirements::from_persistence_identity(&encoded),
            Some(requirements)
        );
        assert!(
            ApplicationWorkflowExecutionRequirements::from_persistence_identity(&format!(
                "{encoded}x"
            ))
            .is_none()
        );
    }
    for encoded in [
        "2:none",
        "",
        "1:approval:999999999999999999999999:1:x",
        "1:approval:0:0:",
        "1:approval:01:1:ab",
    ] {
        assert!(
            ApplicationWorkflowExecutionRequirements::from_persistence_identity(encoded).is_none()
        );
    }
}
