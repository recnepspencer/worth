use std::marker::PhantomData;
use worth_proof::{
    AdmitExecutionReadyRecipeTransition, AssumptionBasis, ContextualTransition, CurrentValidity,
    ExecuteReadyRecipeTransition, ExecutedRecipe, ExecutionReadinessContext, ExecutionReadyRecipe,
    FreshnessScopedBasis, LowerRecipeTransition, Lowered, Performed, Recipe,
    RecipeResolutionContext, ResolveRecipeTransition, Resolved, Transition, Unresolved,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PrototypeOrigin {
    SourceRecompute,
    DependencyCommit,
    StructuralRecompute,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PrototypeWork {
    target_slot: u32,
    origin: PrototypeOrigin,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PrototypeBinding {
    graph_instance: u64,
    dependency_revision: u64,
    readiness_epoch: u64,
}

type CurrentBasis = FreshnessScopedBasis<CurrentValidity, AssumptionBasis<PrototypeBinding>>;
type ResolvedPrototypeRecipe = Recipe<Resolved, PrototypeWork, CurrentBasis>;
type LoweredPrototypeBatch = Recipe<Lowered, PrototypeWork, CurrentBasis>;
type ReadyPrototypeBatch = ExecutionReadyRecipe<PrototypeWork, CurrentBasis>;
type ExecutedPrototypeBatch = ExecutedRecipe<PrototypeWork, CurrentBasis>;

struct AdmittedSourceRecompute(Recipe<Unresolved, PrototypeWork>);
struct PreparedDirectInvalidation(Recipe<Unresolved, PrototypeWork>);
struct AdmittedStructuralRecompute(Recipe<Unresolved, PrototypeWork>);

worth_proof::authority_marker!(SourceRecomputeAuthority);
worth_proof::authority_marker!(DependencyCommitAuthority);
worth_proof::authority_marker!(StructuralRecomputeAuthority);
worth_proof::capability_marker!(TopologyLoweringCapability);
worth_proof::authority_marker!(ReadinessAdmissionAuthority);

struct OwnerResolved<Auth>(ResolvedPrototypeRecipe, PhantomData<Auth>);
type ResolvedSourceRecompute = OwnerResolved<SourceRecomputeAuthority>;
type ResolvedDependencyCommit = OwnerResolved<DependencyCommitAuthority>;
type ResolvedStructuralRecompute = OwnerResolved<StructuralRecomputeAuthority>;

impl AdmittedSourceRecompute {
    fn new(target_slot: u32) -> Self {
        Self(unresolved_work(
            target_slot,
            PrototypeOrigin::SourceRecompute,
        ))
    }
}

impl PreparedDirectInvalidation {
    fn new(target_slot: u32) -> Self {
        Self(unresolved_work(
            target_slot,
            PrototypeOrigin::DependencyCommit,
        ))
    }
}

impl AdmittedStructuralRecompute {
    fn new(target_slot: u32) -> Self {
        Self(unresolved_work(
            target_slot,
            PrototypeOrigin::StructuralRecompute,
        ))
    }
}

struct PublishPrototypeDirectCommit;
impl worth_proof::ActionMarker for PublishPrototypeDirectCommit {}

type PerformedPrototypeDirectCommit =
    Performed<PublishPrototypeDirectCommit, DependencyCommitAuthority, u32>;

struct CommittedDirectInvalidation {
    prepared: PreparedDirectInvalidation,
    publication: PerformedPrototypeDirectCommit,
}

#[derive(Default)]
struct PrototypeCommitLedger(Vec<u32>);

fn publish_direct_commit(
    prepared: PreparedDirectInvalidation,
    ledger: &mut PrototypeCommitLedger,
) -> CommittedDirectInvalidation {
    let target = prepared.0.payload().target_slot;
    ledger.0.push(target);
    let publication = Performed::record(&DependencyCommitAuthority::witness(), target);
    CommittedDirectInvalidation {
        prepared,
        publication,
    }
}

fn resolve_source(
    admitted: AdmittedSourceRecompute,
    binding: PrototypeBinding,
) -> ResolvedSourceRecompute {
    resolve_recipe(admitted.0, binding, SourceRecomputeAuthority::witness())
}

fn resolve_dependency(
    committed: CommittedDirectInvalidation,
    binding: PrototypeBinding,
) -> ResolvedDependencyCommit {
    assert_eq!(
        committed.prepared.0.payload().target_slot,
        *committed.publication.outcome()
    );
    resolve_recipe(
        committed.prepared.0,
        binding,
        DependencyCommitAuthority::witness(),
    )
}

fn resolve_structural(
    admitted: AdmittedStructuralRecompute,
    binding: PrototypeBinding,
) -> ResolvedStructuralRecompute {
    resolve_recipe(admitted.0, binding, StructuralRecomputeAuthority::witness())
}

fn resolve_recipe<Auth: worth_proof::AuthorityMarker>(
    recipe: Recipe<Unresolved, PrototypeWork>,
    binding: PrototypeBinding,
    authority: worth_proof::AuthorityWitness<Auth>,
) -> OwnerResolved<Auth> {
    let resolved = ResolveRecipeTransition
        .transition(recipe, RecipeResolutionContext::new(binding, authority))
        .into_value();
    OwnerResolved(resolved, PhantomData)
}

enum ResolvedOriginWork {
    Source(ResolvedSourceRecompute),
    Dependency(ResolvedDependencyCommit),
    Structural(ResolvedStructuralRecompute),
}

fn lower(resolved: ResolvedOriginWork) -> LoweredPrototypeBatch {
    let recipe = match resolved {
        ResolvedOriginWork::Source(resolved) => resolved.0,
        ResolvedOriginWork::Dependency(resolved) => resolved.0,
        ResolvedOriginWork::Structural(resolved) => resolved.0,
    };
    LowerRecipeTransition::new(TopologyLoweringCapability::witness())
        .transition(recipe)
        .into_value()
}

fn admit_ready(lowered: LoweredPrototypeBatch) -> ReadyPrototypeBatch {
    AdmitExecutionReadyRecipeTransition
        .transition(
            lowered,
            ExecutionReadinessContext::new((), ReadinessAdmissionAuthority::witness()),
        )
        .into_value()
}

fn execute_ready(ready: ReadyPrototypeBatch) -> ExecutedPrototypeBatch {
    ExecuteReadyRecipeTransition.transition(ready).into_value()
}

fn unresolved_work(target_slot: u32, origin: PrototypeOrigin) -> Recipe<Unresolved, PrototypeWork> {
    Recipe::new(PrototypeWork {
        target_slot,
        origin,
    })
}

fn binding() -> PrototypeBinding {
    PrototypeBinding {
        graph_instance: 11,
        dependency_revision: 13,
        readiness_epoch: 17,
    }
}

#[test]
fn existing_proof_surface_supports_owner_specific_origins_and_ready_only_execution() {
    let mut ledger = PrototypeCommitLedger::default();
    let source =
        ResolvedOriginWork::Source(resolve_source(AdmittedSourceRecompute::new(7), binding()));
    let dependency = ResolvedOriginWork::Dependency(resolve_dependency(
        publish_direct_commit(PreparedDirectInvalidation::new(7), &mut ledger),
        binding(),
    ));
    let structural = ResolvedOriginWork::Structural(resolve_structural(
        AdmittedStructuralRecompute::new(7),
        binding(),
    ));

    assert_eq!(ledger.0, [7]);
    for resolved in [source, dependency, structural] {
        let executed = execute_ready(admit_ready(lower(resolved)));
        assert_eq!(executed.strong_basis().value(), &binding());
    }
}
