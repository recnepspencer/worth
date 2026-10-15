//! Complete action meaning and macro lowering through the production authoring owner.
use super::*;
use crate::application_program::*;

mod duplicates;
mod world;
use world::*;

struct Program<const REVERSED: bool, const PROVIDER: bool = true>;
impl<const REVERSED: bool, const PROVIDER: bool> ApplicationProgramDefinition<MutationSchema>
    for Program<REVERSED, PROVIDER>
{
    type Contributions = ();
    type Outputs = ApplicationProgramOutputs<Root>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("test.action.program.v1");

    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        let source = if REVERSED {
            crate::worth_query_feature_spec!(
                root(MutationSchema, Source);
                action(RenameBinding, Delivery) {
                    external_input(Provider);
                    evaluated_requirement(Rule);
                    correspondence(Correspondence);
                    locality_and_change(Scope, Change);
                };
                provides(Export);
            )
        } else {
            let action = ApplicationFeatureSpec::root::<MutationSchema, Source>()
                .action::<RenameBinding, Delivery>()
                .locality_and_change::<Scope, Change>()
                .correspondence::<Correspondence>()
                .evaluated_requirement::<Rule>();
            let parent = if PROVIDER {
                action.external_input::<Provider>().finish_action()
            } else {
                action.finish_action()
            };
            parent.provides::<Export>().finish()
        };
        vec![
            source,
            ApplicationFeatureSpec::root::<MutationSchema, Target>().finish(),
        ]
    }
}

#[test]
fn combined_action_retains_every_contract_and_delivery_in_one_declaration() {
    let program = ApplicationProgramAuthoring::<MutationSchema, Program<false>>::begin()
        .validated_program()
        .unwrap();
    let [action] = program.actions() else {
        panic!("one complete action must be appended")
    };
    assert_eq!(action.binding(), RenameBinding::IDENTITY);
    assert_eq!(
        action.mutation_binding_type(),
        Some(TypeId::of::<RenameBinding>())
    );
    assert_eq!(action.composition_instance(), ApplicationRootComposition::PATH);
    assert_eq!(action.feature(), Source::IDENTITY);
    assert!(action.required_output_source());
    let delivery = action.delivery().unwrap().fixed_output().unwrap();
    assert_eq!(delivery.identity(), Delivery::IDENTITY);
    assert_eq!(delivery.root_type(), TypeId::of::<Root>());
    assert_eq!(action.locality().unwrap().identity(), Scope::IDENTITY);
    assert_eq!(
        action.change_shape().unwrap().posture(),
        ApplicationChangePosture::Replace
    );
    assert_eq!(
        action.correspondence().unwrap().identity(),
        Correspondence::IDENTITY
    );
    assert_eq!(
        action.evaluated_requirement().unwrap().identity(),
        Rule::IDENTITY
    );
    assert_eq!(action.external_input().unwrap().identity(), Provider::IDENTITY);
    assert_eq!(
        action.workflow_execution_requirements(),
        RenameBinding::workflow_execution_requirements().as_ref()
    );
}

#[test]
fn reordered_macro_clauses_preserve_canonical_action_and_program_meaning() {
    let direct = ApplicationProgramAuthoring::<MutationSchema, Program<false>>::begin()
        .validated_program()
        .unwrap();
    let block = ApplicationProgramAuthoring::<MutationSchema, Program<true>>::begin()
        .validated_program()
        .unwrap();
    assert_eq!(direct.actions(), block.actions());
    assert_eq!(direct.normalized_manifest(), block.normalized_manifest());
    assert_eq!(direct.revision(), block.revision());
}

#[test]
fn changed_metadata_changes_canonical_revision_without_changing_delivery() {
    let complete = ApplicationProgramAuthoring::<MutationSchema, Program<false>>::begin()
        .validated_program()
        .unwrap();
    let removed = ApplicationProgramAuthoring::<MutationSchema, Program<false, false>>::begin()
        .validated_program()
        .unwrap();
    assert_eq!(
        complete.actions()[0].delivery(),
        removed.actions()[0].delivery()
    );
    assert_ne!(complete.actions(), removed.actions());
    assert_ne!(complete.revision(), removed.revision());
}

struct NativeProgram<const PROVIDER: bool>;
impl<const PROVIDER: bool> ApplicationProgramDefinition<MutationSchema>
    for NativeProgram<PROVIDER>
{
    type Contributions = ();
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("test.action.native.v1");

    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![if PROVIDER {
            ApplicationFeatureSpec::root::<MutationSchema, Source>()
                .native_action::<RenameBinding>()
                .external_input::<Provider>()
                .finish_action()
                .finish()
        } else {
            crate::worth_query_feature_spec!(
                root(MutationSchema, Source);
                native_action(RenameBinding) { evaluated_requirement(Rule); };
            )
        }]
    }
}

#[test]
fn native_rule_and_provider_are_independent_and_empty_blocks_are_valid() {
    let rule = ApplicationProgramAuthoring::<MutationSchema, NativeProgram<false>>::begin()
        .validated_program()
        .unwrap();
    let provider = ApplicationProgramAuthoring::<MutationSchema, NativeProgram<true>>::begin()
        .validated_program()
        .unwrap();
    assert!(rule.actions()[0].evaluated_requirement().is_some());
    assert!(rule.actions()[0].external_input().is_none());
    assert!(provider.actions()[0].evaluated_requirement().is_none());
    assert!(provider.actions()[0].external_input().is_some());
    assert!(provider.actions()[0].delivery().is_none());
    assert!(!provider.actions()[0].required_output_source());

    let native = crate::worth_query_feature_spec!(
        root(MutationSchema, Source); native_action(RenameBinding) {};
    );
    let ordinary = crate::worth_query_feature_spec!(
        root(MutationSchema, Source); action(RenameBinding, Delivery) {};
    );
    assert_eq!(native.actions().len(), 1);
    assert!(native.actions()[0].delivery().is_none());
    assert_eq!(ordinary.actions().len(), 1);
    assert!(ordinary.actions()[0].required_output_source());
}
