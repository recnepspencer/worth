use crate::application_program::{
    ApplicationFeature, ApplicationFeatureInputLeaf, ApplicationFeatureSpec,
};

crate::worth_query_application_schema! {
    schema FeatureSpecMacroSchema {
        owner: feature_spec_macro_test,
        version: (1, 0),
        members: |schema| { schema }
    }
}

struct Feature;

impl ApplicationFeature<FeatureSpecMacroSchema> for Feature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "worth.query.test.feature-spec-macro.v1";
}

#[test]
fn feature_spec_macro_lowers_through_the_canonical_builder() {
    FeatureSpecMacroSchema::declaration().expect("test schema should declare");
    let declared = crate::worth_query_feature_spec!(root(FeatureSpecMacroSchema, Feature););
    let canonical = ApplicationFeatureSpec::root::<FeatureSpecMacroSchema, Feature>().finish();
    assert_eq!(declared, canonical);
}
