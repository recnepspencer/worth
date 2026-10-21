//! One real topology installation with scoped required producer membership.
use super::*;
worth_query_decl::facade::worth_query_application! {
    pub(super) JointSchema {
        owner: "worth.query.certification.conditional-native-action",
        version: (1, 0), contributions: [TopologyContribution],
    }
}
impl TopologySchemaBinding for JointSchema {
    const PLANAR_REQUIRES_APPLICATION_PROGRAM: bool = true;
}

pub(super) struct JointProgram<const MODE: u8>;
impl<const MODE: u8> ApplicationProgramDefinition<JointSchema> for JointProgram<MODE> {
    type Contributions = <JointSchema as ApplicationSchemaComposition>::Contributions;
    // Deliberately no output roots: an output-source association cannot hide
    // missing concrete action membership in this joint assembly court.
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleList<
        ApplicationRuleAt<
            ApplicationSharedRuleRef<JointSchema, PositivePlanarTurn>,
            ApplicationCommitBoundary,
        >,
        ApplicationRuleLeaf,
    >;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("conditional-native-joint-program");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        let feature = ApplicationFeatureSpec::root::<JointSchema, PlanarSourceFeature>()
            .native_action::<PlanarSourceAdjustmentBinding<JointSchema>>()
            .finish_action()
            .conditional_operation::<PublishChain>()
            .conditional_operation::<PublishFinalPlanarOutput>()
            .conditional_operation::<PreserveFinalPlanarOutput>()
            .conditional_operation::<PublishAlternatePlanarOutput>();
        let feature = match MODE {
            0 | 5 | 7 => feature
                .conditional_native_action::<PlanarMutationBinding<JointSchema>>()
                .finish_action(),
            1 => feature.conditional_operation::<MutatePlanar>(),
            2 => feature
                .native_action::<PlanarMutationBinding<JointSchema>>()
                .finish_action()
                .conditional_operation::<MutatePlanar>(),
            3 => feature
                .action::<PlanarMutationBinding<JointSchema>, OrdinaryDelivery>()
                .finish_action()
                .conditional_operation::<MutatePlanar>(),
            4 => feature,
            6 => feature
                .conditional_native_action::<PlanarEditBinding<JointSchema>>()
                .finish_action(),
            _ => unreachable!("closed fixture declaration modes"),
        };
        let feature = if MODE == 5 {
            feature.conditional_operation::<EditPlanar>()
        } else {
            feature
        };
        let feature = if MODE == 7 {
            feature
                .conditional_native_action::<custody_source::ValueSourceBinding<JointSchema>>()
                .finish_action()
        } else {
            feature
        };
        vec![feature.finish()]
    }
}
struct OrdinaryDelivery;
impl ApplicationActionDelivery<JointSchema, PlanarMutationBinding<JointSchema>>
    for OrdinaryDelivery
{
    type Mode = ApplicationRetainedCommit;
    const IDENTITY: &'static str = "conditional-native-ordinary-retained";
}
