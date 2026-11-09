//! Real separately admitted predecessor and successor schema/program meanings.
use std::num::NonZeroU64;
use worth_query_declaration::facade::application_schema::fragment::{self, Fragment, NoBindings};
use worth_query_declaration::facade::{application_program::*, application_schema::*};
use worth_query_installation::facade::*;

pub(super) struct Before;
pub(super) struct After;
pub(super) struct Record;
struct SourceFeature;
struct TargetFeature;
struct TargetRule;
struct SourceProgram;
struct TargetProgram;
struct CurrentProgram;

impl fragment::Entity for Record {
    const NAME: &'static str = "Record";
    type Aspects = ();
    type Relations = ();
}
impl Fragment for Before {
    const NAME: &'static str = "Before";
    type Bindings = NoBindings;
    type Members = (Record,);
}
impl Fragment for After {
    const NAME: &'static str = "After";
    type Bindings = NoBindings;
    type Members = (Record,);
}
impl ApplicationSchema for Before {
    const OWNER: &'static str = "transition-requirements";
    const NAME: &'static str = "Before";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        fragment::declare(Self)
    }
}
impl ApplicationSchema for After {
    const OWNER: &'static str = "transition-requirements";
    const NAME: &'static str = "After";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        fragment::declaration_builder(Self)?
            .invariant(ApplicationInvariantDefinition::new(
                TargetRule::reference(),
                ApplicationInvariantExecutionPoint::CommitBoundary,
                RelationalInvariantWorkBudget::new(Some(NonZeroU64::new(64).unwrap())),
                ApplicationInvariantOperationalContract::new(
                    ApplicationInvariantEnforcement::BlockCommit,
                    [ApplicationInvariantGroup::SchemaCompliance],
                    [ApplicationInvariantScopeTarget::Entity("Record".into())],
                    [ApplicationInvariantScopeTarget::Entity("Record".into())],
                    "TargetRuleProvider",
                    ApplicationInvariantCostPosture::Touched,
                ),
            ))
            .build()
    }
}
impl ApplicationInvariantMarkerIdentity<After> for TargetRule {
    const IDENTIFIER: &'static str = "TargetRule";
    const MAJOR: u16 = 1;
    const MINOR: u16 = 0;
}
impl ApplicationFeature<Before> for SourceFeature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "worth.query.transition.source.v1";
}
impl ApplicationFeature<After> for TargetFeature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "worth.query.transition.target.v1";
}
impl ApplicationProgramDefinition<Before> for SourceProgram {
    type Contributions = ();
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.transition.before.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![ApplicationFeatureSpec::root::<Before, SourceFeature>().finish()]
    }
}
impl ApplicationProgramDefinition<After> for TargetProgram {
    type Contributions = ();
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleList<
        ApplicationRuleAt<ApplicationSharedRuleRef<After, TargetRule>, ApplicationCommitBoundary>,
        ApplicationRuleLeaf,
    >;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.transition.after.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![ApplicationFeatureSpec::root::<After, TargetFeature>().finish()]
    }
}
impl ApplicationProgramDefinition<After> for CurrentProgram {
    type Contributions = ();
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.transition.current.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        TargetProgram::feature_specs()
    }
}

pub(super) struct Pair {
    pub before: WorthQueryInstalledApplicationSchema<Before>,
    pub after: WorthQueryInstalledApplicationSchema<After>,
    pub source_roster: WorthQueryProgramSupportRoster<Before>,
    pub target_roster: WorthQueryProgramSupportRoster<After>,
    pub source: ApplicationProgramRevision,
    pub target: ApplicationProgramRevision,
    pub current: ApplicationProgramRevision,
}

pub(super) fn pair() -> Pair {
    let before = install::<Before>();
    let after = install::<After>();
    let source = ApplicationProgramAuthoring::<Before, SourceProgram>::begin()
        .validated_program()
        .unwrap();
    let target = ApplicationProgramAuthoring::<After, TargetProgram>::begin()
        .validated_program()
        .unwrap();
    let current = ApplicationProgramAuthoring::<After, CurrentProgram>::begin()
        .validated_program()
        .unwrap();
    let source_roster = WorthQueryProgramSupportAdmission::for_installed_schema(&before)
        .support(&source)
        .unwrap()
        .close()
        .unwrap();
    let target_roster = WorthQueryProgramSupportAdmission::for_installed_schema(&after)
        .support(&target)
        .unwrap()
        .support(&current)
        .unwrap()
        .close()
        .unwrap();
    Pair {
        before,
        after,
        source_roster,
        target_roster,
        source: *source.revision(),
        target: *target.revision(),
        current: *current.revision(),
    }
}

pub(super) fn install<S: ApplicationSchema>() -> WorthQueryInstalledApplicationSchema<S> {
    let declaration = S::declaration().unwrap();
    let package =
        WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(S::OWNER, 1, 0))
            .application_schema(declaration.clone())
            .validate()
            .unwrap();
    let admitted =
        WorthQueryInstallationAdmissionProfile::new("transition-requirements", "installed-pair")
            .admit(package)
            .unwrap();
    WorthQueryInstalledPackageIndex::build(
        WorthQueryInstallationRuntimeIdentity::fresh(),
        WorthQueryInstallationGeneration::initial(),
        [admitted],
    )
    .unwrap()
    .bind_application_schema(declaration)
    .unwrap()
}
