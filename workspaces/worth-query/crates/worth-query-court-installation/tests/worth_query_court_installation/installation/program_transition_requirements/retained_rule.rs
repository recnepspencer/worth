//! Distinct genuine schema declarations retain a rule key with different scope.
use worth_query_declaration::facade::{application_program::*, application_schema::*};
use worth_query_installation::facade::*;

struct ChangedSchema;
struct MatchingSchema;
struct Record;
struct ExtraRecord;
struct RetainedRule;
struct Feature;
struct ChangedProgram;
struct MatchingProgram;

impl ApplicationEntityMarkerIdentity<ChangedSchema> for Record {
    const IDENTIFIER: &'static str = "Record";
}
impl ApplicationEntityMarkerIdentity<MatchingSchema> for Record {
    const IDENTIFIER: &'static str = "Record";
}
impl ApplicationEntityMarkerIdentity<ChangedSchema> for ExtraRecord {
    const IDENTIFIER: &'static str = "EarlierRecord";
}
impl ApplicationEntityMarkerIdentity<MatchingSchema> for ExtraRecord {
    const IDENTIFIER: &'static str = "EarlierRecord";
}
impl ApplicationInvariantMarkerIdentity<ChangedSchema> for RetainedRule {
    const IDENTIFIER: &'static str = "TargetRule";
    const MAJOR: u16 = 1;
    const MINOR: u16 = 0;
}
impl ApplicationInvariantMarkerIdentity<MatchingSchema> for RetainedRule {
    const IDENTIFIER: &'static str = "TargetRule";
    const MAJOR: u16 = 1;
    const MINOR: u16 = 0;
}
impl ApplicationSchema for ChangedSchema {
    const OWNER: &'static str = "transition-requirements";
    const NAME: &'static str = "Changed";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        declaration::<Self>("EarlierRecord")
    }
}
impl ApplicationSchema for MatchingSchema {
    const OWNER: &'static str = "transition-requirements";
    const NAME: &'static str = "Matching";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        declaration::<Self>("Record")
    }
}

fn declaration<S: ApplicationSchema>(
    scope: &str,
) -> Result<ApplicationSchemaDeclaration<S>, ApplicationSchemaDeclarationDenial>
where
    Record: ApplicationEntityMarkerIdentity<S>,
    ExtraRecord: ApplicationEntityMarkerIdentity<S>,
    RetainedRule: ApplicationInvariantMarkerIdentity<S>,
{
    ApplicationSchemaDeclarationBuilder::<S>::for_schema()
        .entity(ApplicationEntityRef::<S, Record>::from_schema_identifier(
            "Record",
        ))
        .entity(ApplicationEntityRef::<S, ExtraRecord>::from_schema_identifier("EarlierRecord"))
        .invariant(ApplicationInvariantDefinition::new(
            RetainedRule::reference(),
            ApplicationInvariantExecutionPoint::CommitBoundary,
            RelationalInvariantWorkBudget::new(Some(std::num::NonZeroU64::new(64).unwrap())),
            ApplicationInvariantOperationalContract::new(
                ApplicationInvariantEnforcement::BlockCommit,
                [ApplicationInvariantGroup::SchemaCompliance],
                [ApplicationInvariantScopeTarget::Entity(scope.into())],
                [ApplicationInvariantScopeTarget::Entity(scope.into())],
                "TargetRuleProvider",
                ApplicationInvariantCostPosture::Touched,
            ),
        ))
        .build()
}

impl ApplicationFeature<ChangedSchema> for Feature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "worth.query.transition.target.v1";
}
impl ApplicationFeature<MatchingSchema> for Feature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "worth.query.transition.target.v1";
}
impl ApplicationProgramDefinition<ChangedSchema> for ChangedProgram {
    type Contributions = ();
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleList<
        ApplicationRuleAt<
            ApplicationSharedRuleRef<ChangedSchema, RetainedRule>,
            ApplicationCommitBoundary,
        >,
        ApplicationRuleLeaf,
    >;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.transition.after.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![ApplicationFeatureSpec::root::<ChangedSchema, Feature>().finish()]
    }
}
impl ApplicationProgramDefinition<MatchingSchema> for MatchingProgram {
    type Contributions = ();
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleList<
        ApplicationRuleAt<
            ApplicationSharedRuleRef<MatchingSchema, RetainedRule>,
            ApplicationCommitBoundary,
        >,
        ApplicationRuleLeaf,
    >;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.transition.after.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![ApplicationFeatureSpec::root::<MatchingSchema, Feature>().finish()]
    }
}

fn predecessor<S: ApplicationSchema, P: ApplicationProgramDefinition<S>>() -> (
    WorthQueryInstalledApplicationSchema<S>,
    WorthQueryProgramSupportRoster<S>,
    ApplicationProgramRevision,
) {
    let installed = super::fixture::install::<S>();
    let program = ApplicationProgramAuthoring::<S, P>::begin()
        .validated_program()
        .unwrap();
    let roster = WorthQueryProgramSupportAdmission::for_installed_schema(&installed)
        .support(&program)
        .unwrap()
        .close()
        .unwrap();
    (installed, roster, *program.revision())
}

#[test]
fn installed_pair_refuses_changed_applicability_under_a_retained_rule_key() {
    let p = super::fixture::pair();
    let (before, roster, revision) = predecessor::<ChangedSchema, ChangedProgram>();
    assert_eq!(
        roster.entry(&revision).unwrap().rules(),
        p.target_roster.entry(&p.target).unwrap().rules()
    );
    let denial = p
        .target_roster
        .adoption_requirements_from_installed_predecessor(
            &p.after, &roster, &before, &revision, &p.target, 65_536,
        )
        .expect_err("a retained key cannot erase changed installed applicability");
    let WorthQueryProgramAdoptionRequirementsDenial::InstalledRuleContractChanged { rule } = denial
    else {
        panic!("expected the installed contract distinction, got {denial:?}");
    };
    assert_eq!(rule.identity(), "TargetRule");
    assert_eq!((rule.major(), rule.minor()), (1, 0));

    let (unchanged, roster, revision) = predecessor::<MatchingSchema, MatchingProgram>();
    assert_ne!(unchanged.binding_identity(), p.after.binding_identity());
    let admitted = p
        .target_roster
        .adoption_requirements_from_installed_predecessor(
            &p.after, &roster, &unchanged, &revision, &p.target, 65_536,
        )
        .expect("a separate installation can preserve the exact retained rule contract");
    assert!(admitted.semantically_equivalent());
    assert!(admitted.validation_scopes().is_empty());
    assert!(admitted.added_rules().is_empty());
}
