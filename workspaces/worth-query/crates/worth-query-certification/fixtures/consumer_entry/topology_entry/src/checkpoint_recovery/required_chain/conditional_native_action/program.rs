//! One real topology installation with scoped required producer membership.
use super::*;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct JointSchema;
impl worth_query_decl::facade::application_schema::fragment::Fragment for JointSchema {
    const NAME: &'static str = "JointSchema";
    type Bindings = worth_query_decl::facade::application_schema::fragment::NoBindings;
    type Members = (TopologyContribution,);
}
impl worth_query_decl::facade::application_schema::ApplicationSchema for JointSchema {
    const OWNER: &'static str = "worth.query.certification.conditional-native-action";
    const NAME: &'static str = "JointSchema";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;
    fn declaration() -> Result<
        worth_query_decl::facade::application_schema::ApplicationSchemaDeclaration<Self>,
        worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationDenial,
    > {
        worth_query_decl::facade::application_schema::fragment::declare(Self)
    }
    fn visit_correction<
        V: worth_query_decl::facade::application_aftermath::ApplicationCorrectionVisitor<Self>,
    >(
        source: std::any::TypeId,
        visitor: V,
    ) -> V::Output {
        worth_query_decl::facade::application_schema::fragment::visit_correction(
            Self, source, visitor,
        )
    }
}
impl worth_query_decl::facade::application_schema::ApplicationSchemaComposition for JointSchema {
    type Contributions = (TopologyContribution,);
}

impl TopologySchemaBinding for JointSchema {
    type PrincipalFragment = crate::PrincipalRecords;
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
            .required_pair::<PlanarFinalFeatureRoles<JointSchema, PlanarSourceFeature>>()
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
