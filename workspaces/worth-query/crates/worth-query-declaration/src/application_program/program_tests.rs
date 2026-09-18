use super::{
    ApplicationConnectionIdentity, ApplicationConnectionRef, ApplicationEvaluatedRequirement,
    ApplicationEvaluatedRequirementRule, ApplicationExternalInputProvider,
    ApplicationExternalInputResolution, ApplicationFeature, ApplicationFeatureInputLeaf,
    ApplicationFeatureInputList, ApplicationFeatureSpec, ApplicationInputPort,
    ApplicationNoOutputGraph, ApplicationOccurrenceConnectionBinding, ApplicationOutputGraph,
    ApplicationOutputLeaf, ApplicationOutputPort, ApplicationProgramAuthoring,
    ApplicationProgramDefinition, ApplicationProgramIdentity, ApplicationProgramOutputs,
    ApplicationRuleLeaf,
};
use crate::application_schema::{
    ApplicationOperationMarkerIdentity, ApplicationSchema, ApplicationSchemaDeclaration,
    ApplicationSchemaDeclarationDenial, ApplicationStructuredValueBinding,
};

mod locality;

struct TestSchema;
struct FlatFeature;
struct Input;
struct InputBinding;
struct Operation;
struct ConsumerFeature;
struct Export;
struct Import;
struct Connection;
struct RequirementRule;
struct TestExternalProvider {
    revision: Option<u64>,
    valid: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TestExternalDenial {
    Removed,
    Changed,
    Invalid,
}

impl ApplicationSchema for TestSchema {
    const OWNER: &'static str = "worth.query.tests";
    const NAME: &'static str = "feature-spec";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;

    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        unreachable!("program authoring does not construct a schema declaration")
    }
}

impl ApplicationFeature<TestSchema> for FlatFeature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "worth.query.tests.flat-feature.v1";
}

impl ApplicationFeature<TestSchema> for ConsumerFeature {
    type Inputs = ApplicationFeatureInputList<Import, ApplicationFeatureInputLeaf>;
    const IDENTITY: &'static str = "worth.query.tests.consumer-feature.v1";
}

impl ApplicationOutputPort<TestSchema, FlatFeature> for Export {
    type Value = InputBinding;
    const IDENTITY: &'static str = "worth.query.tests.flat-feature.export.v1";
}

impl ApplicationInputPort<TestSchema, ConsumerFeature> for Import {
    type Value = InputBinding;
    const IDENTITY: &'static str = "worth.query.tests.consumer-feature.import.v1";
    const REQUIRED: bool = true;
}

impl ApplicationConnectionIdentity for Connection {
    const IDENTITY: &'static str = "worth.query.tests.flat-consumer.v1";
}

impl ApplicationOccurrenceConnectionBinding<TestSchema, FlatFeature, ConsumerFeature>
    for Connection
{
}

type FlatConnection =
    ApplicationConnectionRef<TestSchema, FlatFeature, Export, ConsumerFeature, Import, Connection>;

impl ApplicationStructuredValueBinding for InputBinding {
    type Value = Input;
    const IDENTITY_NAME: &'static str = "worth.query.tests.flat-feature.input.v1";

    fn validate(
        _: &Self::Value,
    ) -> Result<(), crate::application_schema::ApplicationValueValidationDenial> {
        Ok(())
    }
}

impl ApplicationOperationMarkerIdentity<TestSchema> for Operation {
    type InputBinding = InputBinding;
    const IDENTIFIER: &'static str = "worth.query.tests.flat-feature.operation.v1";
}

impl ApplicationEvaluatedRequirementRule<TestSchema, Operation> for RequirementRule {
    const IDENTITY: &'static str = "worth.query.tests.required-label.v1";
    type Context = bool;
    type Requirement = &'static str;
    type Finding = &'static str;

    fn evaluate(
        required: &Self::Context,
    ) -> ApplicationEvaluatedRequirement<Self::Requirement, Self::Finding> {
        if *required {
            ApplicationEvaluatedRequirement::required("label", "mode-requires-label")
        } else {
            ApplicationEvaluatedRequirement::not_required("mode-does-not-require-label")
        }
    }
}

impl ApplicationExternalInputProvider<TestSchema, Operation> for TestExternalProvider {
    const IDENTITY: &'static str = "worth.query.tests.external-input.v1";
    type Selection = &'static str;
    type Values = u64;
    type Revision = u64;
    type Provenance = &'static str;
    type Denial = TestExternalDenial;

    fn resolve(
        &self,
        _: &Self::Selection,
    ) -> Result<ApplicationExternalInputResolution<u64, u64, &'static str>, Self::Denial> {
        if !self.valid {
            return Err(TestExternalDenial::Invalid);
        }
        let revision = self.revision.ok_or(TestExternalDenial::Removed)?;
        Ok(ApplicationExternalInputResolution::new(
            revision,
            revision,
            "neutral-provider",
        ))
    }

    fn validate_revision(
        &self,
        _: &Self::Selection,
        revision: &Self::Revision,
    ) -> Result<(), Self::Denial> {
        match self.revision {
            None => Err(TestExternalDenial::Removed),
            Some(_) if !self.valid => Err(TestExternalDenial::Invalid),
            Some(current) if current != *revision => Err(TestExternalDenial::Changed),
            Some(_) => Ok(()),
        }
    }
}

struct FlatProgram;

impl ApplicationProgramDefinition<TestSchema> for FlatProgram {
    type Contributions = ();
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.tests.flat-program.v1");

    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![ApplicationFeatureSpec::root::<TestSchema, FlatFeature>()
            .conditional_operation::<Operation>()
            .finish()]
    }
}

struct DuplicateProgram;
struct ConnectedProgram;
struct MissingExportProgram;
struct RequirementProgram;

impl ApplicationProgramDefinition<TestSchema> for DuplicateProgram {
    type Contributions = ();
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.tests.duplicate-program.v1");

    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![
            ApplicationFeatureSpec::root::<TestSchema, FlatFeature>().finish(),
            ApplicationFeatureSpec::root::<TestSchema, FlatFeature>()
                .conditional_operation::<Operation>()
                .finish(),
        ]
    }
}

impl ApplicationProgramDefinition<TestSchema> for ConnectedProgram {
    type Contributions = ();
    type Outputs =
        ApplicationProgramOutputs<ApplicationOutputGraph<FlatConnection, ApplicationOutputLeaf>>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.tests.connected-program.v1");

    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![
            ApplicationFeatureSpec::root::<TestSchema, FlatFeature>()
                .provides::<Export>()
                .finish(),
            ApplicationFeatureSpec::root::<TestSchema, ConsumerFeature>().finish(),
        ]
    }
}

impl ApplicationProgramDefinition<TestSchema> for MissingExportProgram {
    type Contributions = ();
    type Outputs = <ConnectedProgram as ApplicationProgramDefinition<TestSchema>>::Outputs;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.tests.missing-export-program.v1");

    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![
            ApplicationFeatureSpec::root::<TestSchema, FlatFeature>().finish(),
            ApplicationFeatureSpec::root::<TestSchema, ConsumerFeature>().finish(),
        ]
    }
}

impl ApplicationProgramDefinition<TestSchema> for RequirementProgram {
    type Contributions = ();
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.tests.requirement-program.v1");

    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![ApplicationFeatureSpec::root::<TestSchema, FlatFeature>()
            .conditional_operation_with_requirement_and_external_input::<
                Operation,
                RequirementRule,
                TestExternalProvider,
            >()
            .finish()]
    }
}

#[test]
fn flat_feature_spec_enters_the_canonical_program_with_its_action() {
    let program = ApplicationProgramAuthoring::<TestSchema, FlatProgram>::begin()
        .validated_program()
        .expect("flat feature program validates");

    assert_eq!(program.features().len(), 1);
    assert_eq!(program.features()[0].identity(), FlatFeature::IDENTITY);
    assert_eq!(program.actions().len(), 1);
    assert_eq!(program.actions()[0].feature(), FlatFeature::IDENTITY);
    assert_eq!(program.actions()[0].binding(), Operation::IDENTIFIER);
}

#[test]
fn duplicate_feature_specs_are_rejected() {
    let denial = match ApplicationProgramAuthoring::<TestSchema, DuplicateProgram>::begin()
        .validated_program()
    {
        Ok(_) => panic!("duplicate feature specs must be rejected"),
        Err(denial) => denial,
    };

    assert_eq!(
        denial.kind(),
        super::ApplicationProgramValidationDenialKind::DuplicateFeature
    );
    assert_eq!(denial.subject(), FlatFeature::IDENTITY);
}

#[test]
fn declared_export_enters_the_validated_feature_dependency() {
    let program = ApplicationProgramAuthoring::<TestSchema, ConnectedProgram>::begin()
        .validated_program()
        .expect("typed feature export validates");

    assert_eq!(
        program.features()[0].outputs()[0].identity(),
        Export::IDENTITY
    );
    assert_eq!(program.connections()[0].source_port(), Export::IDENTITY);
}

#[test]
fn connection_cannot_consume_an_unprovided_feature_export() {
    let denial = match ApplicationProgramAuthoring::<TestSchema, MissingExportProgram>::begin()
        .validated_program()
    {
        Ok(_) => panic!("an unprovided source port must not install"),
        Err(denial) => denial,
    };

    assert_eq!(
        denial.kind(),
        super::ApplicationProgramValidationDenialKind::UndeclaredOutput
    );
    assert_eq!(
        denial.subject(),
        "worth.query.tests.flat-feature.v1.worth.query.tests.flat-feature.export.v1"
    );
}

#[test]
fn evaluated_requirement_uses_one_rule_result_for_guidance_and_enforcement() {
    let program = ApplicationProgramAuthoring::<TestSchema, RequirementProgram>::begin()
        .validated_program()
        .expect("evaluated requirement program validates");
    assert!(program.rules().is_empty());
    let requirement = program.actions()[0]
        .evaluated_requirement()
        .expect("the requirement stays attached to its operation");
    assert_eq!(requirement.identity(), RequirementRule::IDENTITY);
    assert_eq!(
        requirement.rule_type(),
        std::any::TypeId::of::<RequirementRule>()
    );
    let provider = program.actions()[0]
        .external_input()
        .expect("the provider stays attached to its operation");
    assert_eq!(provider.identity(), TestExternalProvider::IDENTITY);

    let required = RequirementRule::evaluate(&true);
    assert_eq!(required.required_input_guidance(), Some(&"label"));
    let denial = required
        .enforce_submission(false)
        .expect_err("the same evaluation denies a missing required input");
    assert_eq!(denial.missing(), &"label");
    assert_eq!(denial.finding(), &"mode-requires-label");
    assert!(required.enforce_submission(true).is_ok());

    let optional = RequirementRule::evaluate(&false);
    assert_eq!(optional.required_input_guidance(), None);
    assert!(optional.enforce_submission(false).is_ok());
}

#[test]
fn external_input_contract_distinguishes_changed_removed_and_invalid_selections() {
    let stable = TestExternalProvider {
        revision: Some(7),
        valid: true,
    };
    let captured = stable.resolve(&"material").unwrap();
    assert_eq!(captured.values(), &7);
    assert_eq!(captured.revision(), &7);
    assert_eq!(captured.provenance(), &"neutral-provider");
    assert!(stable
        .validate_revision(&"material", captured.revision())
        .is_ok());
    assert_eq!(
        TestExternalProvider {
            revision: Some(8),
            valid: true,
        }
        .validate_revision(&"material", captured.revision()),
        Err(TestExternalDenial::Changed)
    );
    assert_eq!(
        TestExternalProvider {
            revision: None,
            valid: true,
        }
        .validate_revision(&"material", captured.revision()),
        Err(TestExternalDenial::Removed)
    );
    assert_eq!(
        TestExternalProvider {
            revision: Some(7),
            valid: false,
        }
        .validate_revision(&"material", captured.revision()),
        Err(TestExternalDenial::Invalid)
    );
}
