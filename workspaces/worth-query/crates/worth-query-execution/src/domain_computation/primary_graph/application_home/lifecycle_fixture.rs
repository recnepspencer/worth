//! A real program open with one committed scalar record for home lifecycle proofs.
use super::ApplicationHome;
use crate::domain_computation::primary_graph::{
    application_installation::{
        program, WorthQueryApplicationLimits, WorthQueryProgramApplicationRuntime,
    },
    WorthQueryApplicationEntityKey, WorthQueryApplicationEntitySeed,
    WorthQueryPrimaryGraphIntegrationHandle,
};
use worth_query_declaration::facade::{
    application_program::*, application_schema::*, authentication::*,
};

worth_query_declaration::worth_query_application! {
    pub HomeSchema {
        owner: "query_home_lifecycle", version: (1, 0),
        contributions: [HomeContribution],
    }
}
worth_query_declaration::worth_query_application_contribution! {
    pub contribution HomeContribution in HomeSchema {
        identity: "worth.query.home.lifecycle.contribution",
        members: |schema| {
            schema.entity(HomeMapping::reference()).entity(HomePrincipal::reference())
                .aspect(HomeMapping::reference(), HomeMappingAspect::reference())
                .aspect(HomePrincipal::reference(), HomePrincipalAspect::reference())
                .field(HomeMapping::reference(), HomeExternalIdentity::reference())
                .field(HomeMapping::reference(), HomeMappingStatus::reference())
                .field(HomePrincipal::reference(), HomePrincipalIdentity::reference())
                .relation(HomeMappingTarget::reference(), HomeMapping::reference(), HomePrincipal::reference())
                .principal_binding(HomePrincipalBinding::reference())
                .entity(HomeRecord::reference())
                .aspect(HomeRecord::reference(), HomeAspect::reference())
                .field(HomeRecord::reference(), HomeValue::reference())
        }
    }
}
worth_query_declaration::worth_query_entity!(pub HomeRecord for HomeSchema);
worth_query_declaration::worth_query_aspect!(
    pub HomeAspect for HomeSchema, HomeRecord;
    identity = AspectIdentity(1), revision = AspectContractRevision(1),
);
worth_query_declaration::worth_query_field!(
    pub HomeValue for HomeSchema, HomeRecord, HomeAspect:
    u64 => U64ApplicationValueBinding, read_write, no_equality
);
struct HomeFeature;
impl ApplicationFeature<HomeSchema> for HomeFeature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "worth.query.home.lifecycle.feature";
}
pub struct HomeProgram;
impl ApplicationProgramDefinition<HomeSchema> for HomeProgram {
    type Contributions = (HomeContribution,);
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.home.lifecycle.program");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![ApplicationFeatureSpec::root::<HomeSchema, HomeFeature>().finish()]
    }
}
pub type Runtime = WorthQueryProgramApplicationRuntime<HomeSchema, HomeProgram>;

pub fn open(home: ApplicationHome) -> Runtime {
    open_with_projection(home).0
}

pub fn open_with_projection(
    home: ApplicationHome,
) -> (
    Runtime,
    super::super::WorthQueryApplicationInvariantProjectionAuthority<HomeSchema>,
) {
    use crate::domain_computation::execution_runtime::*;
    let validated = ApplicationProgramAuthoring::<HomeSchema, HomeProgram>::begin()
        .validated_program()
        .expect("the lifecycle fixture program is declaration-valid");
    let limits = WorthQueryApplicationLimits::new(
        product_world::test_product_world_resources(),
        WorthQueryApplicationCandidateResourceProfile::bounded(1024, 1024 * 1024, 4096)
            .expect("finite fixture candidates"),
        WorthQueryApplicationQueryResourceProfile::bounded(1024 * 1024, 1024 * 1024, 1024, 16)
            .expect("finite fixture queries"),
        worth_signal::facade::runtime::SignalConditionalEvaluationBudget::development(),
    );
    let runtime = program(
        validated,
        HomeSchema::declaration().expect("the lifecycle fixture schema is valid"),
        ((),),
        limits,
    )
    .initial_state(|bootstrap, installed| {
        bootstrap.bind_principal(
            &installed
                .principal_binding(HomePrincipalBinding::reference())
                .expect("the fixture binding is installed"),
            super::super::WorthQueryApplicationPrincipalKey::new("principal")
                .expect("the fixture principal key is nonempty"),
            1_u64,
            WorthQueryExternalPrincipalIdentity::new("https://issuer.example", "home")
                .expect("the fixture identity is valid"),
            WorthQueryPrincipalMappingStatus::Enabled,
        )?;
        bootstrap.bind_entity(
            WorthQueryApplicationEntitySeed::new(
                HomeRecord::reference(),
                WorthQueryApplicationEntityKey::new("record").expect("the fixture key is nonempty"),
            )
            .field(HomeValue::reference(), 42),
        )?;
        Ok(())
    })
    .open(home)
    .expect("the lifecycle fixture home opens");
    let invariant = runtime.runtime().retain_invariant_projection_authority();
    (runtime, invariant)
}
pub fn graph(runtime: &Runtime) -> WorthQueryPrimaryGraphIntegrationHandle {
    runtime
        .runtime()
        .runtime
        .primary_graph()
        .expect("the fixture installs a primary graph")
        .integration_handle()
}
pub fn assert_open(runtime: &Runtime) {
    graph(runtime)
        .with_runtime(|native| assert!(!native.owner_is_sealed()))
        .expect("a refused close returns the same open owner");
}

pub fn candidate(
    runtime: &Runtime,
) -> worth_relational::facade::mvcc::PreparedRelationalCommitCandidate {
    use std::collections::BTreeMap;
    use worth_relational::facade::transactions::{
        AspectFieldPatch, EntityMutationIntent, MutationIntent, UpdateEntityFieldsIntent,
        WorkerIntentBatch,
    };
    let invariant = runtime.runtime().retain_invariant_projection_authority();
    let snapshot = invariant.snapshot().expect("the fixture record projects");
    let id = snapshot
        .entities(HomeRecord::reference())
        .expect("the fixture owner is open")[0]
        .entity_id();
    drop(snapshot);
    graph(runtime)
        .with_query_runtime_mut(|native, layout| {
            let field = HomeValue::reference();
            let locator = layout
                .field_locator(field.entity(), field.aspect(), field.field())
                .expect("the fixture field is installed")
                .clone();
            let fields = AspectFieldPatch::from(BTreeMap::from([(
                locator,
                U64ApplicationValueBinding::encode(&43).expect("the fixture value encodes"),
            )]));
            let (_, basis) = native
                .observe_branch(&native.main_branch_identity())
                .expect("the fixture main branch is admitted");
            let mut transaction = native
                .begin_branch_transaction(
                    &basis,
                    worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
                )
                .expect("the fixture admits a real transaction");
            transaction
                .push_batch(WorkerIntentBatch::new("home-lifecycle-write").push(
                    MutationIntent::Entity(EntityMutationIntent::UpdateFields(
                        UpdateEntityFieldsIntent {
                            entity_id: id,
                            fields,
                        },
                    )),
                ))
                .expect("the fixture write fits its resources");
            native
                .prepare_branch_transaction(transaction)
                .expect("the fixture write prepares")
        })
        .expect("the fixture owner remains open")
}

impl super::super::WorthQueryApplicationContribution<HomeSchema> for HomeContribution {
    type Configuration = ();
    fn configure(
        _: (),
        _: &mut super::super::WorthQueryApplicationContributionSetup<'_, HomeSchema>,
    ) -> Result<(), super::super::WorthQueryPrimaryGraphInstallationDenial> {
        Ok(())
    }
}

worth_query_declaration::worth_query_entity!(pub HomeMapping for HomeSchema);
worth_query_declaration::worth_query_entity!(pub HomePrincipal for HomeSchema);
worth_query_declaration::worth_query_aspect!(pub HomeMappingAspect for HomeSchema, HomeMapping; identity = AspectIdentity(2), revision = AspectContractRevision(1),);
worth_query_declaration::worth_query_aspect!(pub HomePrincipalAspect for HomeSchema, HomePrincipal; identity = AspectIdentity(3), revision = AspectContractRevision(1),);
worth_query_declaration::worth_query_field!(pub HomeExternalIdentity for HomeSchema, HomeMapping, HomeMappingAspect: WorthQueryExternalPrincipalIdentity => WorthQueryExternalPrincipalIdentityBinding, read_only, equality);
worth_query_declaration::worth_query_field!(pub HomeMappingStatus for HomeSchema, HomeMapping, HomeMappingAspect: WorthQueryPrincipalMappingStatus => WorthQueryPrincipalMappingStatusBinding, read_write, equality);
worth_query_declaration::worth_query_field!(pub HomePrincipalIdentity for HomeSchema, HomePrincipal, HomePrincipalAspect: u64 => U64ApplicationValueBinding, read_only, equality);
worth_query_declaration::worth_query_relation!(pub HomeMappingTarget in HomeSchema, HomeMapping => HomePrincipal; integrity = same_context_unbounded_retain_dangling);
worth_query_declaration::worth_query_principal_binding!(pub HomePrincipalBinding in HomeSchema, mapping HomeMapping { identity: HomeExternalIdentity, status: HomeMappingStatus, target: HomeMappingTarget => HomePrincipal, principal_identity: HomePrincipalIdentity });
