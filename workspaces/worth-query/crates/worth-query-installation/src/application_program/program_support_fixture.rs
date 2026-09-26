//! One installed schema carrying two invariant contracts, and the authored
//! programs that declare them, so support admission can be judged against real
//! installed meaning rather than assembled expectations.

use std::num::NonZeroU64;

use worth_query_declaration::facade::application_program::{
    ApplicationCommitBoundary, ApplicationFeature, ApplicationFeatureInputLeaf,
    ApplicationFeatureSpec, ApplicationNoOutputGraph, ApplicationProgramAuthoring,
    ApplicationProgramDefinition, ApplicationProgramIdentity, ApplicationProgramOutputs,
    ApplicationRuleAt, ApplicationRuleLeaf, ApplicationRuleList, ApplicationSharedRuleRef,
    ValidatedApplicationProgram,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationEntityRef, ApplicationInvariantCostPosture, ApplicationInvariantDefinition,
    ApplicationInvariantEnforcement, ApplicationInvariantExecutionPoint, ApplicationInvariantGroup,
    ApplicationInvariantMarkerIdentity, ApplicationInvariantOperationalContract,
    ApplicationInvariantScopeTarget, ApplicationOperationMarkerIdentity, ApplicationOperationRef,
    ApplicationSchema, ApplicationSchemaDeclaration, ApplicationSchemaDeclarationBuilder,
    ApplicationSchemaDeclarationDenial, ApplicationStructuredValueBinding,
    ApplicationValueValidationDenial, RelationalInvariantWorkBudget,
};

use crate::facade::{
    WorthQueryInstallationAdmissionProfile, WorthQueryInstallationGeneration,
    WorthQueryInstallationRuntimeIdentity, WorthQueryInstalledApplicationSchema,
    WorthQueryInstalledPackageIndex, WorthQueryPortableDomainIdentity,
    WorthQueryPortableDomainPackage,
};

pub(super) struct SupportSchema;
pub(super) struct BoundedDimension;
pub(super) struct AuditTrail;
pub(super) struct RaisedBoundedDimension;

struct SupportEntity;
struct BoundedFeature;
struct AuditFeature;
pub(super) struct AdjustInput;
pub(super) struct AdjustInputBinding;
struct AdjustOperation;
pub(super) struct UnknownOperation;

pub(super) const BOUNDED_RULE: &str = "BoundedDimension";
pub(super) const AUDIT_RULE: &str = "AuditTrail";
pub(super) const ADJUST_OPERATION: &str = "AdjustDimension";
pub(super) const UNKNOWN_OPERATION: &str = "RetireDimension";

impl
    worth_query_declaration::facade::application_schema::ApplicationEntityMarkerIdentity<
        SupportSchema,
    > for SupportEntity
{
    const IDENTIFIER: &'static str = "SupportEntity";
}

impl ApplicationInvariantMarkerIdentity<SupportSchema> for BoundedDimension {
    const IDENTIFIER: &'static str = BOUNDED_RULE;
    const MAJOR: u16 = 1;
    const MINOR: u16 = 0;
}

impl ApplicationInvariantMarkerIdentity<SupportSchema> for RaisedBoundedDimension {
    const IDENTIFIER: &'static str = BOUNDED_RULE;
    const MAJOR: u16 = 2;
    const MINOR: u16 = 0;
}

impl ApplicationInvariantMarkerIdentity<SupportSchema> for AuditTrail {
    const IDENTIFIER: &'static str = AUDIT_RULE;
    const MAJOR: u16 = 1;
    const MINOR: u16 = 0;
}

impl ApplicationStructuredValueBinding for AdjustInputBinding {
    type Value = AdjustInput;
    const IDENTITY_NAME: &'static str = "worth.query.installation-test.adjust-dimension";

    fn validate(_: &Self::Value) -> Result<(), ApplicationValueValidationDenial> {
        Ok(())
    }
}

impl ApplicationOperationMarkerIdentity<SupportSchema> for AdjustOperation {
    type InputBinding = AdjustInputBinding;
    const IDENTIFIER: &'static str = ADJUST_OPERATION;
}

impl ApplicationOperationMarkerIdentity<SupportSchema> for UnknownOperation {
    type InputBinding = AdjustInputBinding;
    const IDENTIFIER: &'static str = UNKNOWN_OPERATION;
}

impl ApplicationFeature<SupportSchema> for BoundedFeature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "worth.query.installation-test.bounded-feature.v1";
}

impl ApplicationFeature<SupportSchema> for AuditFeature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "worth.query.installation-test.audit-feature.v1";
}

impl ApplicationSchema for SupportSchema {
    const OWNER: &'static str = "program-support-test";
    const NAME: &'static str = "SupportSchema";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;

    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        ApplicationSchemaDeclarationBuilder::<Self>::for_schema()
            .entity(
                ApplicationEntityRef::<Self, SupportEntity>::from_schema_identifier(
                    "SupportEntity",
                ),
            )
            .operation(
                ApplicationOperationRef::<Self, AdjustOperation, AdjustInput>::from_declaration()
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .invariant(installed_invariant::<BoundedDimension>())
            .invariant(installed_invariant::<AuditTrail>())
            .build()
    }
}

fn installed_invariant<Invariant>() -> ApplicationInvariantDefinition<SupportSchema, Invariant>
where
    Invariant: ApplicationInvariantMarkerIdentity<SupportSchema>,
{
    ApplicationInvariantDefinition::new(
        Invariant::reference(),
        ApplicationInvariantExecutionPoint::CommitBoundary,
        RelationalInvariantWorkBudget::new(NonZeroU64::new(64).expect("64 is nonzero")),
        ApplicationInvariantOperationalContract::new(
            ApplicationInvariantEnforcement::BlockCommit,
            [ApplicationInvariantGroup::SchemaCompliance],
            [ApplicationInvariantScopeTarget::Entity(
                "SupportEntity".to_owned(),
            )],
            [ApplicationInvariantScopeTarget::Entity(
                "SupportEntity".to_owned(),
            )],
            "SupportProvider",
            ApplicationInvariantCostPosture::Touched,
        ),
    )
}

/// Installs the fixture schema through the real package admission path so the
/// invariant catalog under test is the one installation actually compiled.
pub(super) fn installed_support_schema() -> WorthQueryInstalledApplicationSchema<SupportSchema> {
    let package = WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
        SupportSchema::OWNER,
        1,
        0,
    ))
    .application_schema(SupportSchema::declaration().expect("the fixture schema is declarable"))
    .validate()
    .expect("the fixture package is valid");
    let admitted = WorthQueryInstallationAdmissionProfile::new("support", "configuration")
        .admit(package)
        .expect("the fixture package is admissible");
    WorthQueryInstalledPackageIndex::build(
        WorthQueryInstallationRuntimeIdentity::fresh(),
        WorthQueryInstallationGeneration::initial(),
        [admitted],
    )
    .expect("the fixture index builds")
    .bind_application_schema(
        SupportSchema::declaration().expect("the fixture schema is declarable"),
    )
    .expect("the fixture schema binds to its own installation")
}

type BoundedRules = ApplicationRuleList<
    ApplicationRuleAt<
        ApplicationSharedRuleRef<SupportSchema, BoundedDimension>,
        ApplicationCommitBoundary,
    >,
    ApplicationRuleLeaf,
>;

type RaisedRules = ApplicationRuleList<
    ApplicationRuleAt<
        ApplicationSharedRuleRef<SupportSchema, RaisedBoundedDimension>,
        ApplicationCommitBoundary,
    >,
    ApplicationRuleLeaf,
>;

type AuditRules = ApplicationRuleList<
    ApplicationRuleAt<
        ApplicationSharedRuleRef<SupportSchema, AuditTrail>,
        ApplicationCommitBoundary,
    >,
    ApplicationRuleLeaf,
>;

type CompleteRules = ApplicationRuleList<
    ApplicationRuleAt<
        ApplicationSharedRuleRef<SupportSchema, BoundedDimension>,
        ApplicationCommitBoundary,
    >,
    AuditRules,
>;

/// Declares both installed rules, so it owns the catalog on its own.
pub(super) struct CompleteProgram;
/// Declares the bounded rule and acts through the installed operation.
pub(super) struct BoundedProgram;
/// Declares the audit rule only, so it needs a peer to complete the catalog.
pub(super) struct AuditProgram;
/// Declares a rule version this host never installed.
pub(super) struct RaisedRuleProgram;
/// Acts through an operation this host never installed.
pub(super) struct UninstalledActionProgram;
/// Governs nothing.
pub(super) struct FeaturelessProgram;

macro_rules! support_program {
    ($program:ty, $rules:ty, $identity:literal, $specs:expr) => {
        impl ApplicationProgramDefinition<SupportSchema> for $program {
            type Contributions = ();
            type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
            type Rules = $rules;
            const IDENTITY: ApplicationProgramIdentity = ApplicationProgramIdentity::new($identity);

            fn feature_specs() -> Vec<ApplicationFeatureSpec> {
                $specs
            }
        }
    };
}

support_program!(
    CompleteProgram,
    CompleteRules,
    "worth.query.installation-test.complete-program.v1",
    vec![
        ApplicationFeatureSpec::root::<SupportSchema, BoundedFeature>()
            .conditional_operation::<AdjustOperation>()
            .finish()
    ]
);
support_program!(
    BoundedProgram,
    BoundedRules,
    "worth.query.installation-test.bounded-program.v1",
    vec![
        ApplicationFeatureSpec::root::<SupportSchema, BoundedFeature>()
            .conditional_operation::<AdjustOperation>()
            .finish()
    ]
);
support_program!(
    AuditProgram,
    AuditRules,
    "worth.query.installation-test.audit-program.v1",
    vec![ApplicationFeatureSpec::root::<SupportSchema, AuditFeature>().finish()]
);
support_program!(
    RaisedRuleProgram,
    RaisedRules,
    "worth.query.installation-test.raised-rule-program.v1",
    vec![ApplicationFeatureSpec::root::<SupportSchema, BoundedFeature>().finish()]
);
support_program!(
    UninstalledActionProgram,
    ApplicationRuleLeaf,
    "worth.query.installation-test.uninstalled-action-program.v1",
    vec![
        ApplicationFeatureSpec::root::<SupportSchema, BoundedFeature>()
            .conditional_operation::<UnknownOperation>()
            .finish()
    ]
);
support_program!(
    FeaturelessProgram,
    ApplicationRuleLeaf,
    "worth.query.installation-test.featureless-program.v1",
    Vec::new()
);

pub(super) fn validated<Program>() -> ValidatedApplicationProgram<SupportSchema, Program>
where
    Program: ApplicationProgramDefinition<SupportSchema>,
{
    ApplicationProgramAuthoring::<SupportSchema, Program>::begin()
        .validated_program()
        .expect("the fixture programs are declaration-valid")
}
