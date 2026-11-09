use crate::application_program::{
    ApplicationFeature, ApplicationFeatureInputLeaf, ApplicationFeatureSpec,
};

use crate::facade::application_schema::{
    fragment::{self, Fragment, NoBindings},
    ApplicationSchema, ApplicationSchemaDeclaration, ApplicationSchemaDeclarationDenial,
};
struct FeatureSpecMacroSchema;
impl Fragment for FeatureSpecMacroSchema {
    const NAME: &'static str = "FeatureSpecMacroSchema";
    type Bindings = NoBindings;
    type Members = ();
}
impl ApplicationSchema for FeatureSpecMacroSchema {
    const OWNER: &'static str = "feature_spec_macro_test";
    const NAME: &'static str = <Self as Fragment>::NAME;
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        fragment::declare(Self)
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
