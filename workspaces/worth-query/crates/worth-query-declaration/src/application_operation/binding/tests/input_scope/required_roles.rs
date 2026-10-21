//! Custom role projections must pass the real program authoring boundary.
use super::*;
use crate::application_program::*;

struct Feature;
impl ApplicationFeature<MutationSchema> for Feature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "required-pair-feature";
}
struct Family;
struct OtherFamily;
struct Source;
struct OtherSource;
struct InitialProducer;
struct InitialReadiness;
struct PreserveProducer;
struct PreserveReadiness;
struct Roles<const CASE: u8>;

impl<const CASE: u8> ApplicationRequiredFeatureRoles<MutationSchema> for Roles<CASE> {
    type Feature = Feature;
    type InitialMutation = InitialBinding;
    type PreserveMutation = PreserveBinding;

    fn roles() -> [ApplicationRequiredFeatureRoleDeclaration; 2] {
        use ApplicationRequiredFeatureOutput as Output;
        use ApplicationRequiredFeatureRoleDeclaration as Role;
        use ApplicationRequiredFeatureRoleKind as Kind;
        let initial = Output::of::<Family, Source>("family", "source", "body");
        let preserve = match CASE {
            1 => Output::of::<OtherFamily, Source>("family", "source", "body"),
            2 => Output::of::<Family, Source>("other-family", "source", "body"),
            3 => Output::of::<Family, OtherSource>("family", "source", "body"),
            4 => Output::of::<Family, Source>("family", "other-source", "body"),
            5 => Output::of::<Family, Source>("family", "source", "other-role"),
            _ => Output::of::<Family, Source>("family", "source", "body"),
        };
        [
            Role::of::<MutationSchema, InitialBinding, InitialProducer, InitialReadiness>(
                Kind::Initial,
                "initial",
                "initial-ready",
                initial,
            ),
            Role::of::<MutationSchema, PreserveBinding, PreserveProducer, PreserveReadiness>(
                Kind::Preserve,
                "preserve",
                "preserve-ready",
                preserve,
            ),
        ]
    }
}

struct Program<const CASE: u8>;
impl<const CASE: u8> ApplicationProgramDefinition<MutationSchema> for Program<CASE> {
    type Contributions = ();
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("test.required-pair");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![
            crate::worth_query_feature_spec!(root(MutationSchema, Feature);
            required_pair(Roles<CASE>);),
        ]
    }
}

fn reject<const CASE: u8>() {
    let denied = match ApplicationProgramAuthoring::<MutationSchema, Program<CASE>>::begin()
        .validated_program()
    {
        Err(denied) => denied,
        Ok(_) => panic!("mismatched required pair {CASE} passed program admission"),
    };
    assert_eq!(
        denied.kind(),
        ApplicationProgramValidationDenialKind::InvalidRequiredFeatureRole
    );
    assert_eq!(denied.subject(), Feature::IDENTITY);
}

#[test]
fn required_roles_preserve_both_typed_and_stable_output_correspondence() {
    let valid = ApplicationProgramAuthoring::<MutationSchema, Program<0>>::begin()
        .validated_program()
        .unwrap();
    assert_eq!(valid.features()[0].required_roles().len(), 2);
    reject::<1>();
    reject::<2>();
    reject::<3>();
    reject::<4>();
    reject::<5>();
}
