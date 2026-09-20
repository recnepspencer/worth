mod canonical_slot;
mod meaning_axes;
mod semantic_diff;
mod semantic_meaning;
mod work_budget;

use super::super::{
    ApplicationCommitBoundary, ApplicationConnectionIdentity, ApplicationConnectionRef,
    ApplicationFeature, ApplicationFeatureInputLeaf, ApplicationFeatureInputList,
    ApplicationFeatureSpec, ApplicationInputPort, ApplicationMutationSensitive,
    ApplicationNoOutputGraph, ApplicationOccurrenceConnectionBinding, ApplicationOutputGraph,
    ApplicationOutputLeaf, ApplicationOutputPort, ApplicationProgramAuthoring,
    ApplicationProgramDefinition, ApplicationProgramIdentity, ApplicationProgramOutputs,
    ApplicationProgramRevision, ApplicationRuleAt, ApplicationRuleLeaf, ApplicationRuleList,
    ApplicationSharedRuleRef,
};
use crate::application_schema::{
    ApplicationInvariantMarkerIdentity, ApplicationOperationMarkerIdentity, ApplicationSchema,
    ApplicationSchemaDeclaration, ApplicationSchemaDeclarationDenial,
    ApplicationStructuredValueBinding, ApplicationValueValidationDenial,
};

struct RevisionSchema;
struct BoundedFeature;
struct AuditFeature;
struct AdjustInput;
struct AdjustInputBinding;
struct AdjustOperation;
struct AuditOperation;
struct BoundedDimensionV1;
struct BoundedDimensionV2;
struct BoundedDimensionV3;
struct BoundedExport;
struct AuditImport;
struct BoundedToAudit;

impl ApplicationSchema for RevisionSchema {
    const OWNER: &'static str = "worth.query.tests";
    const NAME: &'static str = "program-revision";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;

    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        unreachable!("program authoring does not construct a schema declaration")
    }
}

impl ApplicationFeature<RevisionSchema> for BoundedFeature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "worth.query.tests.bounded-feature.v1";
}

impl ApplicationFeature<RevisionSchema> for AuditFeature {
    type Inputs = ApplicationFeatureInputList<AuditImport, ApplicationFeatureInputLeaf>;
    const IDENTITY: &'static str = "worth.query.tests.audit-feature.v1";
}

impl ApplicationOutputPort<RevisionSchema, BoundedFeature> for BoundedExport {
    type Value = AdjustInputBinding;
    const IDENTITY: &'static str = "worth.query.tests.bounded-feature.export.v1";
}

impl ApplicationInputPort<RevisionSchema, AuditFeature> for AuditImport {
    type Value = AdjustInputBinding;
    const IDENTITY: &'static str = "worth.query.tests.audit-feature.import.v1";
    const REQUIRED: bool = false;
}

impl ApplicationConnectionIdentity for BoundedToAudit {
    const IDENTITY: &'static str = "worth.query.tests.bounded-audit.v1";
}

impl ApplicationOccurrenceConnectionBinding<RevisionSchema, BoundedFeature, AuditFeature>
    for BoundedToAudit
{
}

type BoundedAuditConnection = ApplicationConnectionRef<
    RevisionSchema,
    BoundedFeature,
    BoundedExport,
    AuditFeature,
    AuditImport,
    BoundedToAudit,
>;

impl ApplicationStructuredValueBinding for AdjustInputBinding {
    type Value = AdjustInput;
    const IDENTITY_NAME: &'static str = "worth.query.tests.bounded-feature.input.v1";

    fn validate(_: &Self::Value) -> Result<(), ApplicationValueValidationDenial> {
        Ok(())
    }
}

impl ApplicationOperationMarkerIdentity<RevisionSchema> for AdjustOperation {
    type InputBinding = AdjustInputBinding;
    const IDENTIFIER: &'static str = "worth.query.tests.bounded-feature.adjust.v1";
}

impl ApplicationOperationMarkerIdentity<RevisionSchema> for AuditOperation {
    type InputBinding = AdjustInputBinding;
    const IDENTIFIER: &'static str = "worth.query.tests.bounded-feature.audit.v1";
}

impl ApplicationInvariantMarkerIdentity<RevisionSchema> for BoundedDimensionV1 {
    const IDENTIFIER: &'static str = "BoundedDimension";
    const MAJOR: u16 = 1;
    const MINOR: u16 = 0;
}

impl ApplicationInvariantMarkerIdentity<RevisionSchema> for BoundedDimensionV2 {
    const IDENTIFIER: &'static str = "BoundedDimension";
    const MAJOR: u16 = 2;
    const MINOR: u16 = 0;
}

impl ApplicationInvariantMarkerIdentity<RevisionSchema> for BoundedDimensionV3 {
    const IDENTIFIER: &'static str = "BoundedDimension";
    const MAJOR: u16 = 3;
    const MINOR: u16 = 0;
}

type BaselineRules = ApplicationRuleList<
    ApplicationRuleAt<
        ApplicationSharedRuleRef<RevisionSchema, BoundedDimensionV1>,
        ApplicationCommitBoundary,
    >,
    ApplicationRuleLeaf,
>;

type RaisedRules = ApplicationRuleList<
    ApplicationRuleAt<
        ApplicationSharedRuleRef<RevisionSchema, BoundedDimensionV2>,
        ApplicationCommitBoundary,
    >,
    ApplicationRuleLeaf,
>;

type MultipleRuleVersions = ApplicationRuleList<
    ApplicationRuleAt<
        ApplicationSharedRuleRef<RevisionSchema, BoundedDimensionV1>,
        ApplicationCommitBoundary,
    >,
    ApplicationRuleList<
        ApplicationRuleAt<
            ApplicationSharedRuleRef<RevisionSchema, BoundedDimensionV3>,
            ApplicationCommitBoundary,
        >,
        ApplicationRuleLeaf,
    >,
>;

type ShiftedRuleVersions = ApplicationRuleList<
    ApplicationRuleAt<
        ApplicationSharedRuleRef<RevisionSchema, BoundedDimensionV2>,
        ApplicationCommitBoundary,
    >,
    ApplicationRuleList<
        ApplicationRuleAt<
            ApplicationSharedRuleRef<RevisionSchema, BoundedDimensionV3>,
            ApplicationCommitBoundary,
        >,
        ApplicationRuleLeaf,
    >,
>;

/// The baseline rule inventory moved to a different execution point, which is
/// the only declared difference between the two execution-point programs.
type MutationSensitiveRules = ApplicationRuleList<
    ApplicationRuleAt<
        ApplicationSharedRuleRef<RevisionSchema, BoundedDimensionV1>,
        ApplicationMutationSensitive,
    >,
    ApplicationRuleLeaf,
>;

const SHARED_IDENTITY: ApplicationProgramIdentity =
    ApplicationProgramIdentity::new("worth.query.tests.revision-program.v1");

fn bounded_feature_specs() -> Vec<ApplicationFeatureSpec> {
    vec![
        ApplicationFeatureSpec::root::<RevisionSchema, BoundedFeature>()
            .conditional_operation::<AdjustOperation>()
            .finish(),
    ]
}

/// One exporting feature and one importing feature, so the only difference the
/// output-topology programs can express is whether they connect the two.
fn connectable_feature_specs() -> Vec<ApplicationFeatureSpec> {
    vec![
        ApplicationFeatureSpec::root::<RevisionSchema, BoundedFeature>()
            .conditional_operation::<AdjustOperation>()
            .provides::<BoundedExport>()
            .finish(),
        ApplicationFeatureSpec::root::<RevisionSchema, AuditFeature>().finish(),
    ]
}

struct BaselineProgram;
struct RenamedBaselineProgram;
struct RaisedRuleVersionProgram;
struct MultipleRuleVersionsProgram;
struct ShiftedRuleVersionsProgram;
struct AddedActionProgram;
struct AddedFeatureProgram;
struct MutationSensitiveRuleProgram;
struct UnconnectedTopologyProgram;
struct ConnectedTopologyProgram;

impl ApplicationProgramDefinition<RevisionSchema> for BaselineProgram {
    type Contributions = ();
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = BaselineRules;
    const IDENTITY: ApplicationProgramIdentity = SHARED_IDENTITY;

    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        bounded_feature_specs()
    }
}

impl ApplicationProgramDefinition<RevisionSchema> for RenamedBaselineProgram {
    type Contributions = ();
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = BaselineRules;
    const IDENTITY: ApplicationProgramIdentity = SHARED_IDENTITY;

    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        bounded_feature_specs()
    }
}

impl ApplicationProgramDefinition<RevisionSchema> for RaisedRuleVersionProgram {
    type Contributions = ();
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = RaisedRules;
    const IDENTITY: ApplicationProgramIdentity = SHARED_IDENTITY;

    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        bounded_feature_specs()
    }
}

impl ApplicationProgramDefinition<RevisionSchema> for MultipleRuleVersionsProgram {
    type Contributions = ();
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = MultipleRuleVersions;
    const IDENTITY: ApplicationProgramIdentity = SHARED_IDENTITY;

    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        bounded_feature_specs()
    }
}

impl ApplicationProgramDefinition<RevisionSchema> for ShiftedRuleVersionsProgram {
    type Contributions = ();
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ShiftedRuleVersions;
    const IDENTITY: ApplicationProgramIdentity = SHARED_IDENTITY;

    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        bounded_feature_specs()
    }
}

impl ApplicationProgramDefinition<RevisionSchema> for AddedActionProgram {
    type Contributions = ();
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = BaselineRules;
    const IDENTITY: ApplicationProgramIdentity = SHARED_IDENTITY;

    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![
            ApplicationFeatureSpec::root::<RevisionSchema, BoundedFeature>()
                .conditional_operation::<AdjustOperation>()
                .conditional_operation::<AuditOperation>()
                .finish(),
        ]
    }
}

impl ApplicationProgramDefinition<RevisionSchema> for AddedFeatureProgram {
    type Contributions = ();
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = BaselineRules;
    const IDENTITY: ApplicationProgramIdentity = SHARED_IDENTITY;

    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        let mut specs = bounded_feature_specs();
        specs.push(ApplicationFeatureSpec::root::<RevisionSchema, AuditFeature>().finish());
        specs
    }
}

impl ApplicationProgramDefinition<RevisionSchema> for MutationSensitiveRuleProgram {
    type Contributions = ();
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = MutationSensitiveRules;
    const IDENTITY: ApplicationProgramIdentity = SHARED_IDENTITY;

    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        bounded_feature_specs()
    }
}

impl ApplicationProgramDefinition<RevisionSchema> for UnconnectedTopologyProgram {
    type Contributions = ();
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = BaselineRules;
    const IDENTITY: ApplicationProgramIdentity = SHARED_IDENTITY;

    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        connectable_feature_specs()
    }
}

impl ApplicationProgramDefinition<RevisionSchema> for ConnectedTopologyProgram {
    type Contributions = ();
    type Outputs = ApplicationProgramOutputs<
        ApplicationOutputGraph<BoundedAuditConnection, ApplicationOutputLeaf>,
    >;
    type Rules = BaselineRules;
    const IDENTITY: ApplicationProgramIdentity = SHARED_IDENTITY;

    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        connectable_feature_specs()
    }
}

fn revision_of<Program>() -> ApplicationProgramRevision
where
    Program: ApplicationProgramDefinition<RevisionSchema>,
{
    ApplicationProgramAuthoring::<RevisionSchema, Program>::begin()
        .validated_program()
        .expect("the revision fixture programs are declaration-valid")
        .revision()
        .clone()
}

fn manifest_of<Program>() -> super::super::ApplicationProgramManifest
where
    Program: ApplicationProgramDefinition<RevisionSchema>,
{
    ApplicationProgramAuthoring::<RevisionSchema, Program>::begin()
        .validated_program()
        .expect("the revision fixture programs are declaration-valid")
        .normalized_manifest()
}

fn description_of<Program>() -> super::super::ApplicationSemanticDescription
where
    Program: ApplicationProgramDefinition<RevisionSchema>,
{
    ApplicationProgramAuthoring::<RevisionSchema, Program>::begin()
        .validated_program()
        .expect("the semantic fixture program is declaration-valid")
        .semantic_description()
        .clone()
}
