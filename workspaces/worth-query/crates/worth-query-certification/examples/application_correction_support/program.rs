//! A real source action with ordinary completion and no required output root.
use super::{edit::EditBinding, other_edit::OtherBinding, schema::*};
use worth_query_host::facade::declaration::application_program::*;
pub struct Program;
pub struct NumericFeature;
pub struct Completion;
pub struct OtherCompletion;
pub struct PrincipalCompletion;
impl ApplicationActionDelivery<NumericSchema, super::principal_status::Binding>
    for PrincipalCompletion
{
    type Mode = ApplicationSourceCompletion;
    const IDENTITY: &'static str = "numeric-correction.principal-completion.v1";
}
impl ApplicationActionDelivery<NumericSchema, OtherBinding> for OtherCompletion {
    type Mode = ApplicationSourceCompletion;
    const IDENTITY: &'static str = "numeric-correction.other-completion.v1";
}
impl ApplicationFeature<NumericSchema> for NumericFeature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "numeric-correction.feature.v1";
}
impl ApplicationActionDelivery<NumericSchema, EditBinding> for Completion {
    type Mode = ApplicationSourceCompletion;
    const IDENTITY: &'static str = "numeric-correction.completion.v1";
}
pub type Action = ApplicationActionReference<
    NumericSchema,
    ApplicationRootComposition,
    NumericFeature,
    EditBinding,
    Completion,
>;
impl ApplicationProgramDefinition<NumericSchema> for Program {
    type Contributions = (NumericContribution,);
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("numeric-correction.program.v1");
    fn adoption_authorization<
        Visitor: ApplicationProgramAdoptionAuthorizationVisitor<NumericSchema>,
    >(
        visitor: Visitor,
    ) -> Visitor::Output {
        visitor.capability::<super::adoption::AdoptBinding>()
    }
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        features::<Completion, OtherCompletion>()
    }
}

pub struct Successor;
pub struct ChangedInverse;
pub struct ChangedSource;
pub struct ChangedCompletion;
impl ApplicationActionDelivery<NumericSchema, EditBinding> for ChangedCompletion {
    type Mode = ApplicationSourceCompletion;
    const IDENTITY: &'static str = "numeric-correction.changed-source-completion.v1";
}
impl ApplicationProgramDefinition<NumericSchema> for ChangedSource {
    type Contributions = (NumericContribution,);
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("numeric-correction.changed-source.v1");
    fn adoption_authorization<
        Visitor: ApplicationProgramAdoptionAuthorizationVisitor<NumericSchema>,
    >(
        visitor: Visitor,
    ) -> Visitor::Output {
        visitor.capability::<super::adoption::AdoptBinding>()
    }
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        features::<ChangedCompletion, OtherCompletion>()
    }
}
pub struct ChangedOtherCompletion;
impl ApplicationActionDelivery<NumericSchema, OtherBinding> for ChangedOtherCompletion {
    type Mode = ApplicationSourceCompletion;
    const IDENTITY: &'static str = "numeric-correction.changed-other-completion.v1";
}
impl ApplicationProgramDefinition<NumericSchema> for Successor {
    type Contributions = (NumericContribution,);
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("numeric-correction.successor.v1");
    fn adoption_authorization<
        Visitor: ApplicationProgramAdoptionAuthorizationVisitor<NumericSchema>,
    >(
        visitor: Visitor,
    ) -> Visitor::Output {
        visitor.capability::<super::adoption::AdoptBinding>()
    }
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        features::<Completion, OtherCompletion>()
    }
}
impl ApplicationProgramDefinition<NumericSchema> for ChangedInverse {
    type Contributions = (NumericContribution,);
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("numeric-correction.changed-inverse.v1");
    fn adoption_authorization<
        Visitor: ApplicationProgramAdoptionAuthorizationVisitor<NumericSchema>,
    >(
        visitor: Visitor,
    ) -> Visitor::Output {
        visitor.capability::<super::adoption::AdoptBinding>()
    }
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        features::<Completion, ChangedOtherCompletion>()
    }
}
pub fn validated<P: ApplicationProgramDefinition<NumericSchema>>(
) -> ValidatedApplicationProgram<NumericSchema, P> {
    ApplicationProgramAuthoring::<NumericSchema, P>::begin()
        .validated_program()
        .unwrap()
}
fn features<
    S: ApplicationActionDelivery<NumericSchema, EditBinding>,
    D: ApplicationActionDelivery<NumericSchema, OtherBinding>,
>() -> Vec<ApplicationFeatureSpec> {
    vec![
        super::compensation_movement::feature(),
        super::source_output_model::feature(),
        ApplicationFeatureSpec::root::<NumericSchema, NumericFeature>()
            .conditional_operation::<super::calculation::CalculateNumber>()
            .action::<EditBinding, S>()
            .finish_action()
            .action::<OtherBinding, D>()
            .finish_action()
            .action::<super::principal_status::Binding, PrincipalCompletion>()
            .finish_action()
            .finish(),
    ]
}
