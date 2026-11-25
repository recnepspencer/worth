//! Parity for `#[derive(WorthQueryCorrection)]`, `worth_query_application_program!`,
//! `query(...)` registration, and `as ActionAlias` references against manual
//! trait implementations.
use super::super::actions::*;
use super::super::completion_correction::{
    self, CorrectCompletion, DetailsBinding, DetailsQuery, RestoreCompletion, RestoreDoneBinding,
    RestoreDoneInput, RestoredDone,
};
use super::super::program::{
    BoardDefinition, BoardFeature, CompleteAction, Contribution, Program, RenameAction,
    RescheduleAction, RestoreAction,
};
use super::super::reads;
use super::super::relations;
use super::super::schema::*;
use worth_query_decl::facade::application_aftermath::*;
use worth_query_decl::facade::application_identity::ApplicationIdentityNamespace;
use worth_query_decl::facade::application_operation::*;
use worth_query_decl::facade::application_program::*;
use worth_query_decl::facade::application_query::*;
use worth_query_decl::facade::application_schema::fragment::Registration;
use worth_query_decl::facade::application_schema::*;
use worth_query_decl::facade::portable_identity::WorthQueryPortableTypeIdentity;
use worth_query_decl::facade::{worth_query_identity, worth_query_identity_namespace};
use worth_query_host::facade::{
    application_contribution::*, primary_graph::WorthQueryPrimaryGraphInstallationDenial,
    worth_query_application_contribution,
};

const CORRECTION: ApplicationIdentityNamespace =
    worth_query_identity_namespace!("macros.tasks.correct-completion.v1");

mod manual {
    use super::*;

    pub struct ManualCorrectCompletion;
    impl ApplicationCorrectionBinding<Schema> for ManualCorrectCompletion {
        const IDENTITY: &'static str = worth_query_identity!(CORRECTION, binding);
        type Source = CompleteBinding;
        type InverseIntent = RestoreCompletion;
        type Disclosure = DetailsBinding;

        fn recover_source_input(
            bytes: &[u8],
            expected: CanonicalDigestId,
            bounds: ValueDecodeBounds,
        ) -> Result<ApplicationEncodedInput<CompleteInput>, ApplicationRetainedInputDenial>
        {
            ApplicationEncodedInput::decode_retained(bytes, expected, bounds)
        }

        fn recover_inverse_input(
            bytes: &[u8],
            expected: CanonicalDigestId,
            bounds: ValueDecodeBounds,
        ) -> Result<ApplicationEncodedInput<RestoreDoneInput>, ApplicationRetainedInputDenial>
        {
            ApplicationEncodedInput::decode_retained(bytes, expected, bounds)
        }

        fn disclosure_parameters(
            _: &Complete,
        ) -> Result<ApplicationQueryParameterSet<DetailsQuery>, ApplicationCorrectionDerivationDenial>
        {
            Ok(ApplicationQueryParameterSet::new())
        }

        fn source_definition(
        ) -> ApplicationOperationDefinitionBuilder<Schema, CompleteTask, Complete, true, false>
        {
            CompleteTask::reference().definition().no_external_effect()
        }

        fn source_body() -> Option<ApplicationOperationBody<Schema, CompleteTask, Complete>> {
            Some(complete_body())
        }

        fn mechanism() -> DeclaredCorrectionMechanism<Schema> {
            let bytes = BoolApplicationValueBinding::encode(&true)
                .unwrap()
                .semantic_byte_width();
            DeclaredCorrectionMechanism::recorded_inverse_binding::<Self>(
                DeclaredPreImageDemand::new(
                    [DeclaredPreImageLocus::from_field(Done::at("Done"))],
                    bytes,
                ),
            )
        }

        fn derive_correction(
            original: &Complete,
            material: ApplicationCorrectionSourceMaterial<'_, Schema>,
        ) -> Result<RestoreCompletion, ApplicationCorrectionDerivationDenial> {
            let ApplicationCorrectionSourceMaterial::RecordedInverse(preimage) = material else {
                return Err(ApplicationCorrectionDerivationDenial::UnsupportedSourceMeaning);
            };
            let done = decode_preimage_field(preimage, Done::at("Done"))?;
            let _ = (original, done);
            Err(ApplicationCorrectionDerivationDenial::UnsupportedSourceMeaning)
        }

        fn inverse_key(key: &str) -> Result<u64, ApplicationValueEncodeDenial> {
            key.parse()
                .map_err(|_| ApplicationValueEncodeDenial::CodecRejected {
                    binding_identity: WorthQueryPortableTypeIdentity::declared(Self::IDENTITY),
                    reason: "invalid inverse idempotency key",
                })
        }

        fn visit<V: ApplicationCorrectionVisitor<Schema>>(
            dispatch: ApplicationCorrectionDispatch<Schema, Self, V>,
        ) -> V::Output {
            dispatch.conventional()
        }
    }

    pub struct ManualFeature;
    impl ApplicationFeature<Schema> for ManualFeature {
        type Inputs = ApplicationFeatureInputLeaf;
        const IDENTITY: &'static str = "macros.board.feature.v1";
    }
    worth_query_application_contribution! {
        pub feature ManualDefinition<Schema> for ManualFeature => ManualRuntime {
            schema: |schema| {
                schema
                    .application_query(completion_correction::task_details())
                    .declared_operation(completion_correction::restore_done())
                    .application_query(reads::search())
                    .application_query_binding::<reads::SearchBinding>()
                    .application_query(reads::due())
                    .application_query_binding::<reads::DueBinding>()
                    .application_query(reads::title_search())
                    .application_query_binding::<reads::TitleSearchBinding>()
                    .application_query(relations::outline())
                    .application_query_binding::<relations::OutlineQueryBinding>()
            };
            body_action(RenameBinding, Renamed) { operation: rename_definition(); };
            body_action(RescheduleBinding, Rescheduled) { operation: reschedule_definition(); };
            body_action(RetitleFirstBinding, RetitledFirst) { operation: retitle_first_definition(); };
            body_action_ref(CompleteBinding, Completed) {};
            body_action_ref(RestoreDoneBinding, RestoredDone) {};
        }
    }

    pub struct ManualFeatures;
    impl Registration<Schema> for ManualFeatures {
        fn register(
            schema: ApplicationSchemaDeclarationBuilder<Schema>,
        ) -> Result<ApplicationSchemaDeclarationBuilder<Schema>, ApplicationSchemaDeclarationDenial>
        {
            Ok(ManualDefinition::declare_members(schema))
        }
    }

    pub struct ManualContribution;
    impl ApplicationSchemaContribution<Schema> for ManualContribution {
        const IDENTITY: ApplicationSchemaContributionIdentity =
            ApplicationSchemaContributionIdentity::new("macros.board-contribution.v1");
        type Members = (
            Board,
            PrincipalBinding,
            ManualCorrectCompletion,
            ManualFeatures,
        );
    }
    impl WorthQueryApplicationContribution<Schema> for ManualContribution {
        type Configuration =
            <ManualDefinition as WorthQueryFeatureDefinition<Schema>>::Configuration;
        fn contracts(
            contracts: &mut WorthQueryApplicationContributionContracts<Schema>,
        ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
            ManualDefinition::contracts(contracts)
        }
        fn configure(
            configuration: Self::Configuration,
            setup: &mut WorthQueryApplicationContributionSetup<'_, Schema>,
        ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
            ManualDefinition::configure(configuration, setup)
        }
    }

    pub struct ManualProgram;
    impl ApplicationProgramDefinition<Schema> for ManualProgram {
        type Contributions = (ManualContribution,);
        type Outputs = ApplicationProgramOutputs<
            <ManualDefinition as ApplicationFeatureDefinition<Schema>>::Outputs,
        >;
        type Rules = ApplicationRuleLeaf;
        const IDENTITY: ApplicationProgramIdentity =
            ApplicationProgramIdentity::new("macros.program.v1");
        fn feature_specs() -> Vec<ApplicationFeatureSpec> {
            ManualDefinition::feature_specs()
        }
    }
}

struct EmptyPreImage;
impl ApplicationCorrectionPreImage<Schema> for EmptyPreImage {
    fn field(
        &self,
        _locus: &DeclaredPreImageLocus<Schema>,
    ) -> Option<&worth_foundational::facade::AspectValue> {
        None
    }
}

struct NoOutputs;
impl ApplicationCorrectionSourceOutputs<Schema> for NoOutputs {
    fn field(
        &self,
        _field: &ApplicationCorrectionOutputField<Schema>,
    ) -> Result<&worth_foundational::facade::AspectValue, ApplicationCorrectionSourceFieldDenial>
    {
        Err(ApplicationCorrectionSourceFieldDenial::Refused)
    }
}

#[test]
fn derived_correction_and_collapsed_program_match_manual_declarations_and_aliases() {
    let derived_binding = DeclaredCorrectionBinding::of::<Schema, CorrectCompletion>()
        .expect("derived correction binding describes");
    let manual_binding = DeclaredCorrectionBinding::of::<Schema, manual::ManualCorrectCompletion>()
        .expect("manual correction binding describes");
    assert_eq!(derived_binding, manual_binding);
    assert_eq!(
        <CorrectCompletion as ApplicationCorrectionBinding<Schema>>::mechanism(),
        <manual::ManualCorrectCompletion as ApplicationCorrectionBinding<Schema>>::mechanism()
    );
    assert!(<CorrectCompletion as ApplicationCorrectionBinding<Schema>>::source_body().is_some());

    // Typed denials on missing pre-image, wrong source material, and malformed inverse key.
    let original = Complete {
        task: 3,
        done: true,
    };
    assert_eq!(
        <CorrectCompletion as ApplicationCorrectionBinding<Schema>>::derive_correction(
            &original,
            ApplicationCorrectionSourceMaterial::RecordedInverse(&EmptyPreImage),
        )
        .err(),
        Some(ApplicationCorrectionDerivationDenial::MissingPreImage)
    );
    assert_eq!(
        <CorrectCompletion as ApplicationCorrectionBinding<Schema>>::derive_correction(
            &original,
            ApplicationCorrectionSourceMaterial::Compensation(&NoOutputs),
        )
        .err(),
        Some(ApplicationCorrectionDerivationDenial::UnsupportedSourceMeaning)
    );
    assert!(
        <CorrectCompletion as ApplicationCorrectionBinding<Schema>>::inverse_key("bad").is_err()
    );
    assert_eq!(
        <CorrectCompletion as ApplicationCorrectionBinding<Schema>>::inverse_key("42"),
        Ok(42)
    );

    // Collapsed program matches manual 5-trait program revision and feature specs.
    let schema = Schema::declaration().expect("schema declares");
    let derived_program = ApplicationProgramAuthoring::<Schema, Program>::begin()
        .with_schema(&schema)
        .validated_program()
        .expect("collapsed program validates");
    let manual_program = ApplicationProgramAuthoring::<Schema, manual::ManualProgram>::begin()
        .with_schema(&schema)
        .validated_program()
        .expect("manual program validates");
    assert_eq!(derived_program.revision(), manual_program.revision());
    let derived_specs = <BoardDefinition as ApplicationFeatureDefinition<Schema>>::feature_specs();
    let manual_specs =
        <manual::ManualDefinition as ApplicationFeatureDefinition<Schema>>::feature_specs();
    assert_eq!(derived_specs.len(), manual_specs.len());
    assert_eq!(
        derived_specs[0].feature().identity(),
        manual_specs[0].feature().identity()
    );
    assert_eq!(
        derived_specs[0].feature().composition_instance(),
        manual_specs[0].feature().composition_instance()
    );
    assert_eq!(derived_specs[0].actions(), manual_specs[0].actions());

    // Generated `as ActionAlias` types match manual `ApplicationActionReference`.
    type ManualRestore = ApplicationActionReference<
        Schema,
        ApplicationRootComposition,
        BoardFeature,
        RestoreDoneBinding,
        RestoredDone,
    >;
    assert_eq!(RestoreAction::selector(), ManualRestore::selector());
    assert_eq!(
        RenameAction::selector().binding(),
        <RenameBinding as ApplicationMutationBinding<Schema>>::IDENTITY
    );
    assert_eq!(
        RescheduleAction::selector().binding(),
        <RescheduleBinding as ApplicationMutationBinding<Schema>>::IDENTITY
    );
    assert_eq!(
        CompleteAction::selector().binding(),
        <CompleteBinding as ApplicationMutationBinding<Schema>>::IDENTITY
    );
    assert_eq!(
        <Contribution as ApplicationSchemaContribution<Schema>>::IDENTITY,
        <manual::ManualContribution as ApplicationSchemaContribution<Schema>>::IDENTITY
    );
}
