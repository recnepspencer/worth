//! Repeated genuine actions quote shared delivery meaning once.
use super::*;
use std::marker::PhantomData;
mod graph;
mod projection;

crate::worth_query_mutation_binding!(@binding
    SecondBinding for RenameInput, schema MutationSchema,
    identity "test.action.second.v1", input RenameInputBinding, operation RenameOperation,
    result RenameResultBinding, idempotency u64, identity "test.action.second-key.v1",
    decision RenameDecision, denial RenameDenialBinding,
    handler identity "test.action.second-handler.v1", program required,
    workflow_authority , workflow_requirements , inspection_disclosure , outputs RenameOutputs,
    principal PrincipalBinding, mapping ExternalMapping, principal_entity Account,
    principal_identity u64, identity_binding crate::application_schema::U64ApplicationValueBinding,
    scope_binding crate::application_operation::ApplicationMutationFieldScope<AccountIdField>,
    scope_field_type AccountIdField,
    scope_field AccountIdField::at("AccountIdField"),
);

struct Fixed<const DIFFERENT: bool>;
impl<B: ApplicationMutationBinding<MutationSchema>, const DIFFERENT: bool>
    ApplicationActionDelivery<MutationSchema, B> for Fixed<DIFFERENT>
{
    type Mode = ApplicationFixedDeliveryMode<Root, ApplicationNoProjection>;
    const IDENTITY: &'static str = if DIFFERENT {
        "test.action.distinct-delivery.v1"
    } else {
        "test.action.fixed-delivery.v1"
    };
}
impl<B: ApplicationMutationBinding<MutationSchema>, const DIFFERENT: bool>
    ApplicationFixedActionDelivery<MutationSchema, B> for Fixed<DIFFERENT>
{
    type Root = Root;
    type Projection = ApplicationNoProjection;
}

struct Shared<const DIFFERENT: bool, const REVERSED: bool>;
impl<const DIFFERENT: bool, const REVERSED: bool> ApplicationProgramDefinition<MutationSchema>
    for Shared<DIFFERENT, REVERSED>
{
    type Contributions = ();
    type Outputs = ApplicationProgramOutputs<Root>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("test.action.shared-deliveries.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        let source = ApplicationFeatureSpec::root::<MutationSchema, Source>();
        let source = if REVERSED {
            source
                .action::<SecondBinding, Fixed<DIFFERENT>>()
                .finish_action()
                .action::<RenameBinding, Delivery>()
                .finish_action()
        } else {
            source
                .action::<RenameBinding, Delivery>()
                .finish_action()
                .action::<SecondBinding, Fixed<DIFFERENT>>()
                .finish_action()
        };
        vec![
            source.provides::<Export>().finish(),
            ApplicationFeatureSpec::root::<MutationSchema, Target>().finish(),
        ]
    }
}

fn records(manifest: &ApplicationProgramManifest, family: &str) -> Vec<String> {
    let prefix = format!("{family}|");
    manifest
        .records()
        .iter()
        .filter(|record| record.starts_with(&prefix))
        .cloned()
        .collect()
}

#[test]
fn separate_actions_share_full_delivery_and_connection_meaning_without_losing_membership() {
    let program = ApplicationProgramAuthoring::<MutationSchema, Shared<false, false>>::begin()
        .validated_program()
        .unwrap();
    let manifest = program.normalized_manifest();
    assert_eq!(program.actions().len(), 2);
    assert_eq!(program.connections().len(), 1);
    let actions = records(&manifest, "delivery");
    assert_eq!(actions.len(), 2);
    assert!(actions
        .iter()
        .any(|record| record.contains(RenameBinding::IDENTITY)));
    assert!(actions
        .iter()
        .any(|record| record.contains(SecondBinding::IDENTITY)));
    assert_eq!(records(&manifest, "delivery-definition-v1").len(), 1);
    assert_eq!(records(&manifest, "delivery-connection-v1").len(), 1);
    assert_eq!(
        actions[0].split("|definition=").nth(1),
        actions[1].split("|definition=").nth(1)
    );
    let reversed = ApplicationProgramAuthoring::<MutationSchema, Shared<false, true>>::begin()
        .validated_program()
        .unwrap();
    assert_eq!(manifest, reversed.normalized_manifest());
    assert_eq!(program.revision(), reversed.revision());
}

#[test]
fn distinct_delivery_meanings_share_only_their_identical_connection_vocabulary() {
    let program = ApplicationProgramAuthoring::<MutationSchema, Shared<true, false>>::begin()
        .validated_program()
        .unwrap();
    let manifest = program.normalized_manifest();
    let definitions = records(&manifest, "delivery-definition-v1");
    assert_eq!(definitions.len(), 2);
    assert!(definitions
        .iter()
        .any(|record| record.contains("test.action.distinct-delivery.v1")));
    assert!(definitions
        .iter()
        .any(|record| record.contains("test.action.fixed-delivery.v1")));
    assert_eq!(records(&manifest, "delivery-connection-v1").len(), 1);
    let actions = records(&manifest, "delivery");
    assert_ne!(
        actions[0].split("|definition=").nth(1),
        actions[1].split("|definition=").nth(1)
    );
    let shared = ApplicationProgramAuthoring::<MutationSchema, Shared<false, false>>::begin()
        .validated_program()
        .unwrap();
    assert_ne!(program.revision(), shared.revision());
}

struct Discovered;
type DiscoveredRoot = ApplicationDiscoveredOutputGraph<
    ApplicationConnectionRef<MutationSchema, Source, Export, Target, Import, Connection>,
    ApplicationOutputLeaf,
>;
impl ApplicationActionDelivery<MutationSchema, RenameBinding> for Discovered {
    type Mode = ApplicationDiscoveredDeliveryMode<DiscoveredRoot, ApplicationNoProjection>;
    const IDENTITY: &'static str = "test.action.fixed-delivery.v1";
}
impl ApplicationDiscoveredActionDelivery<MutationSchema, RenameBinding> for Discovered {
    type Root = DiscoveredRoot;
    type Projection = ApplicationNoProjection;
}
struct Single<D>(PhantomData<D>);
impl<D> ApplicationProgramDefinition<MutationSchema> for Single<D>
where
    D: ApplicationActionDelivery<MutationSchema, RenameBinding>,
    D::Mode: ApplicationActionOutputRoots<MutationSchema>,
{
    type Contributions = ();
    type Outputs =
        ApplicationProgramOutputs<<D::Mode as ApplicationActionOutputRoots<MutationSchema>>::Roots>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("test.action.single-delivery.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![
            ApplicationFeatureSpec::root::<MutationSchema, Source>()
                .action::<RenameBinding, D>()
                .finish_action()
                .provides::<Export>()
                .finish(),
            ApplicationFeatureSpec::root::<MutationSchema, Target>()
                .provides::<graph::NextOutput>()
                .finish(),
            ApplicationFeatureSpec::root::<MutationSchema, graph::Last>().finish(),
        ]
    }
}
#[test]
fn mode_is_canonical_but_concrete_delivery_implementation_is_not() {
    let fixed = ApplicationProgramAuthoring::<MutationSchema, Single<Delivery>>::begin()
        .validated_program()
        .unwrap();
    let same_meaning = ApplicationProgramAuthoring::<MutationSchema, Single<Fixed<false>>>::begin()
        .validated_program()
        .unwrap();
    assert_ne!(fixed.actions(), same_meaning.actions());
    assert_eq!(
        fixed.normalized_manifest(),
        same_meaning.normalized_manifest()
    );
    assert_eq!(fixed.revision(), same_meaning.revision());
    let discovered = ApplicationProgramAuthoring::<MutationSchema, Single<Discovered>>::begin()
        .validated_program()
        .unwrap();
    assert_eq!(fixed.connections(), discovered.connections());
    assert_eq!(
        records(&fixed.normalized_manifest(), "delivery-connection-v1"),
        records(&discovered.normalized_manifest(), "delivery-connection-v1")
    );
    assert_ne!(
        fixed.normalized_manifest(),
        discovered.normalized_manifest()
    );
    assert_ne!(fixed.revision(), discovered.revision());
}
