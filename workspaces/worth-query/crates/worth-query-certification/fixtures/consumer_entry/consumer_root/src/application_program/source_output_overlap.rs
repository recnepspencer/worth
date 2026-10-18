//! Native source/action overlap exercises required-output membership validation.
use super::*;

impl ApplicationProgramDefinition<ConsumerSchema> for RequiredSourceAsActionProgram {
    type Contributions = <ConsumerSchema as worth_query_decl::facade::application_schema::ApplicationSchemaComposition>::Contributions;
    type Outputs = <ConsumerProgram as ApplicationProgramDefinition<ConsumerSchema>>::Outputs;
    type Rules = ConsumerRules;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.certification.source-action-overlap.v2");

    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        let mut specs = features::consumer_feature_specs();
        let source = specs
            .iter()
            .position(|spec| {
                spec.feature().identity()
                    == <PlanarSourceFeature as ApplicationFeature<ConsumerSchema>>::IDENTITY
            })
            .expect("the source feature is present");
        specs.remove(source);
        specs.push(
            ApplicationFeatureSpec::root::<ConsumerSchema, PlanarSourceFeature>()
                .provides::<PlanarBodyOutput>()
                .native_action::<PlanarEditBinding<ConsumerSchema>>()
                .finish_action()
                .native_action::<PriorCycleAdjustmentBinding<ConsumerSchema>>()
                .finish_action()
                .native_action::<VertexReplacementBinding<ConsumerSchema>>()
                .finish_action()
                .native_action::<PlanarSourceAdjustmentBinding<ConsumerSchema>>()
                .finish_action()
                .finish(),
        );
        specs
    }
}
