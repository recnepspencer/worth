use super::{
    ApplicationActionLeaf, ApplicationFeature, ApplicationFeatureInputLeaf, ApplicationFeatureLeaf,
    ApplicationFeatureList, ApplicationFeatureRef, ApplicationFeatureSpec,
    ApplicationNoOutputGraph, ApplicationProgramAuthoring, ApplicationProgramDefinition,
    ApplicationProgramIdentity, ApplicationProgramOutputs, ApplicationRuleLeaf,
};
use crate::application_schema::{
    ApplicationOperationMarkerIdentity, ApplicationSchema, ApplicationSchemaDeclaration,
    ApplicationSchemaDeclarationDenial, ApplicationStructuredValueBinding,
};

struct TestSchema;
struct FlatFeature;
struct Input;
struct InputBinding;
struct Operation;

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

struct FlatProgram;

impl ApplicationProgramDefinition<TestSchema> for FlatProgram {
    type Contributions = ();
    type Actions = ApplicationActionLeaf;
    type Features = ApplicationFeatureLeaf;
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

impl ApplicationProgramDefinition<TestSchema> for DuplicateProgram {
    type Contributions = ();
    type Actions = ApplicationActionLeaf;
    type Features = ApplicationFeatureList<
        ApplicationFeatureRef<TestSchema, FlatFeature>,
        ApplicationFeatureLeaf,
    >;
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.tests.duplicate-program.v1");

    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        FlatProgram::feature_specs()
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
fn migrated_feature_cannot_survive_in_the_legacy_inventory() {
    let denial = match ApplicationProgramAuthoring::<TestSchema, DuplicateProgram>::begin()
        .validated_program()
    {
        Ok(_) => panic!("duplicate legacy and flat feature must be rejected"),
        Err(denial) => denial,
    };

    assert_eq!(
        denial.kind(),
        super::ApplicationProgramValidationDenialKind::DuplicateFeature
    );
    assert_eq!(denial.subject(), FlatFeature::IDENTITY);
}
