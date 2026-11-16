//! A zero-result secondary recipe still requires its selected installed graph.
use super::super::super::courts::native_head;
use super::*;
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;

// This fixture program explicitly uses its existing genuine Model-a NAV grant
// to authorize program adoption; no extra grant, principal or capability is seeded.
pub(super) struct Adoption;
impl ApplicationProgramAdoptionBinding<Schema> for Adoption {
    const IDENTITY: &'static str = "owned-outputs.conditional-adoption";
    type Capability = super::super::super::super::navigation::Navigate;
    type Input = query::model::Read;
    type InputBinding = query::model::ReadInput;
    type Operation = super::super::super::super::navigation::SelectSnapshot;
    type PrincipalBinding = Actor;
    type Mapping = Mapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        Schema,
        Actor,
        Mapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        Actor::reference()
    }
    fn input(_: ApplicationProgramAdoptionContext<'_>) -> query::model::Read {
        query::model::Read("a".into())
    }
}
// Same canonical connection meaning, different admitted nominal root shapes.
// An empty pair adds no connection and requires no state migration.
type OtherRetire = ApplicationDiscoveredOutputGraph<
    RetireConnection,
    (ApplicationOutputLeaf, ApplicationOutputLeaf),
>;
type OtherRestore = ApplicationDiscoveredOutputGraph<
    RestoreConnection,
    (ApplicationOutputLeaf, ApplicationOutputLeaf),
>;
struct DifferentConditional;
impl ApplicationProgramDefinition<Schema> for DifferentConditional {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<(OtherRetire, OtherRestore)>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("owned-outputs.different-conditional");
    fn adoption_authorization<V: ApplicationProgramAdoptionAuthorizationVisitor<Schema>>(
        visitor: V,
    ) -> V::Output {
        visitor.capability::<Adoption>()
    }
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        let mut specs = hierarchy::Program::feature_specs();
        specs[0] = base::with_envelope_completion_actions(base::entry_builder_with_retirement::<
            Completion,
            Completion,
            Delivery<OtherRestore>,
            Delivery<OtherRetire>,
        >())
        .finish();
        specs
    }
}
#[test]
fn a_secondary_zero_root_recipe_requires_the_selected_program() {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let schema = Schema::declaration().unwrap();
    let initial = ApplicationProgramAuthoring::<Schema, Program>::begin()
        .with_schema(&schema)
        .validated_program()
        .unwrap();
    let other = ApplicationProgramAuthoring::<Schema, DifferentConditional>::begin()
        .with_schema(&schema)
        .validated_program()
        .unwrap();
    let host = in_memory_rostered_program(
        initial, WorthQueryApplicationProgramRoster::new().support(other), schema,
        (world::Configuration { envelope_records: true, ..Default::default() },),
        ApplicationBuildIdentity::parse("91830000-0000-0000-0000-000000005356").unwrap(),
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()),
        |_, graph, installed| {
            world::seed(graph, installed, 1, None)?;
            fixture::seed(graph)
        },
    ).unwrap();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let target = *host
        .supported_program::<DifferentConditional>()
        .unwrap()
        .installed_program()
        .revision();
    let programs = request.programs();
    let requirements = programs.compare(&target).unwrap();
    let prepared = programs
        .adopt(&requirements)
        .unwrap()
        .prepare(1024)
        .unwrap();
    assert!(matches!(
        prepared.publish(host.runtime(), &principal, &scope),
        WorthQueryBranchAdoptionPublicationOutcome::Performed(_)
    ));
    assert_eq!(request.programs().inspect().unwrap().revision(), &target);
    let before = native_head(&host);
    let mut envelope = request
        .edit_envelope()
        .idempotency("secondary-uninstalled-discovery".to_owned())
        .runtime_stages(&host)
        .ordered_authored()
        .unwrap();
    envelope
        .push_stage::<ApplicationNoOutputGraph, _>(fixture::cap::Intent {
            key: "envelope-cap".into(),
            value: 7,
            key_derived_target: false,
        })
        .unwrap();
    envelope
        .push_stage::<RetireTree, _>(member::retire::Intent)
        .unwrap();
    let refused = envelope.execute_performed(ExecutionAllocationPolicy::SystemAllocation);
    assert!(
        refused.is_err(),
        "uninstalled secondary graph must refuse before its handler"
    );
    assert_eq!(native_head(&host), before);
    assert!(request
        .query(query::members::Read("a".into()))
        .execute()
        .unwrap()
        .rows()[0]
        .members
        .iter()
        .any(|member| member.key == "m" && member.value == 5));
}
