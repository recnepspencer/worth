use worth_signal::facade::{
    NodeEvaluationResult, SignalConditionalArtifactReuse, SignalConditionalVersionComparator,
};

use super::super::{exact_mapping, runtime};
use crate::facade::{
    BridgeConditionalComputeProvider, BridgeConditionalCondition, BridgeConditionalContract,
    BridgeConditionalContractParts, BridgeConditionalLocation, BridgeConditionalProviderSemantics,
    BridgeConditionalProviderSet, BridgeConditionalRetentionBudget,
    BridgeOwnedConditionalInstallationRequest,
};

const SOURCE_FREE_DEFINITION_IDENTITY: &str = "query:source-free-budget";
const PROVIDER_STATE_BYTES: usize = 137;
const SEMANTIC_CONTRACT_STATE_BYTES: usize = 211;

struct HeapBackedCompute {
    output: u64,
    retained_state: Box<[u8]>,
}

#[derive(Eq, PartialEq)]
struct HeapBackedSemanticContract(Box<[u8]>);

impl HeapBackedCompute {
    fn new(output: u64) -> Self {
        Self {
            output,
            retained_state: vec![7; PROVIDER_STATE_BYTES].into_boxed_slice(),
        }
    }
}

impl BridgeConditionalProviderSemantics for HeapBackedCompute {
    type SemanticContract = HeapBackedSemanticContract;

    fn semantic_contract(&self) -> Self::SemanticContract {
        HeapBackedSemanticContract(vec![11; SEMANTIC_CONTRACT_STATE_BYTES].into_boxed_slice())
    }

    fn retained_heap_bytes(
        &self,
        semantic_contract: &Self::SemanticContract,
    ) -> Result<
        crate::facade::BridgeConditionalProviderHeapRetention,
        crate::facade::BridgeConditionalProviderRetentionOverflow,
    > {
        Ok(crate::facade::BridgeConditionalProviderHeapRetention::new(
            self.retained_state.len() as u64,
            semantic_contract.0.len() as u64,
        ))
    }
}

impl BridgeConditionalComputeProvider for HeapBackedCompute {
    fn compute(&self, _: &mut dyn std::any::Any) -> Result<NodeEvaluationResult, String> {
        Ok(NodeEvaluationResult::from_version(
            worth_signal::facade::AspectVersion::from_updates([(
                worth_signal::facade::Aspect::new(0),
                self.output,
            )]),
        ))
    }
}

pub(super) fn source_free_contract(identity: &str) -> BridgeConditionalContract {
    BridgeConditionalContract::new(BridgeConditionalContractParts {
        identity: std::sync::Arc::from(identity),
        dependency_count: 0,
        condition_dependency_ordinals: Vec::new(),
        condition: BridgeConditionalCondition::Always,
        dependency_comparator: SignalConditionalVersionComparator::Exact,
        output_comparator: SignalConditionalVersionComparator::Exact,
        artifact_reuse: SignalConditionalArtifactReuse::NotReusable,
    })
}

fn source_free_successor_owner(
    budget: BridgeConditionalRetentionBudget,
) -> (
    crate::facade::BridgeSealedRuntimeAssembly,
    std::sync::Arc<crate::facade::BridgeInstalledConditionalLowering>,
) {
    let mut bridge = runtime(exact_mapping(), Vec::new());
    bridge.policy = bridge.policy.with_conditional_retention(budget);
    let mut builder = super::super::owned_runtime_builder(bridge).unwrap();
    let lowering = builder
        .install_owned_conditional(BridgeOwnedConditionalInstallationRequest {
            contract: source_free_contract(SOURCE_FREE_DEFINITION_IDENTITY),
            location: BridgeConditionalLocation::operation(SOURCE_FREE_DEFINITION_IDENTITY),
            dependencies: Vec::new(),
            providers: BridgeConditionalProviderSet::new().compute(HeapBackedCompute::new(1)),
        })
        .unwrap();
    (builder.seal().unwrap(), lowering)
}

fn source_free_successor_bytes(owner: &crate::facade::BridgeSealedRuntimeAssembly) -> u64 {
    const SHA256_HEX_BYTES: usize = 64;
    const PROJECTION_PREFIX: &str = "bridge-conditional-lowering:sha256:";
    crate::conditional_execution::definition_candidate_oracle::source_free_successor::<
        HeapBackedCompute,
        HeapBackedSemanticContract,
    >(
        owner
            .admitted_signal_basis()
            .observation()
            .branch_id()
            .as_str(),
        SOURCE_FREE_DEFINITION_IDENTITY,
        SOURCE_FREE_DEFINITION_IDENTITY,
        PROJECTION_PREFIX.len() + SHA256_HEX_BYTES,
        PROVIDER_STATE_BYTES,
        SEMANTIC_CONTRACT_STATE_BYTES,
    )
}

fn prepare_source_free_successor(
    owner: &crate::facade::BridgeSealedRuntimeAssembly,
    lowering: &std::sync::Arc<crate::facade::BridgeInstalledConditionalLowering>,
) -> Result<
    crate::facade::BridgePreparedConditionalInstallationExtension,
    crate::facade::BridgeConditionalDenial,
> {
    let predecessor =
        owner.admit_conditional_signal_basis(lowering, owner.admitted_signal_basis())?;
    owner.prepare_owned_conditional_definition_successor(
        &predecessor,
        BridgeOwnedConditionalInstallationRequest {
            contract: source_free_contract(SOURCE_FREE_DEFINITION_IDENTITY),
            location: BridgeConditionalLocation::operation(SOURCE_FREE_DEFINITION_IDENTITY),
            dependencies: Vec::new(),
            providers: BridgeConditionalProviderSet::new().compute(HeapBackedCompute::new(2)),
        },
    )
}

fn source_free_owner_with_relative_byte_budget(
    mut expected_bytes: u64,
    discount: u64,
) -> (
    crate::facade::BridgeSealedRuntimeAssembly,
    std::sync::Arc<crate::facade::BridgeInstalledConditionalLowering>,
    u64,
) {
    for _ in 0..16 {
        let (owner, lowering) = source_free_successor_owner(BridgeConditionalRetentionBudget {
            maximum_retained_definition_candidates: 1,
            maximum_retained_bytes: expected_bytes - discount,
            ..BridgeConditionalRetentionBudget::development()
        });
        let actual_bytes = source_free_successor_bytes(&owner);
        if actual_bytes == expected_bytes {
            return (owner, lowering, actual_bytes);
        }
        expected_bytes = actual_bytes;
    }
    panic!("generated Signal branch identity length did not stabilize");
}

#[test]
fn real_successor_candidate_obeys_independent_exact_byte_oracle() {
    let (probe_owner, probe_lowering) =
        source_free_successor_owner(BridgeConditionalRetentionBudget::development());
    let exact_bytes = source_free_successor_bytes(&probe_owner);
    let probe_candidate = prepare_source_free_successor(&probe_owner, &probe_lowering).unwrap();
    assert_eq!(
        probe_owner
            .conditional_lifecycle_probe()
            .bridge_conditional_retention()
            .unwrap()
            .retained_bytes(),
        exact_bytes,
    );
    drop(probe_candidate);
    assert_eq!(
        probe_owner
            .conditional_lifecycle_probe()
            .bridge_conditional_retention()
            .unwrap()
            .retained_bytes(),
        0,
    );

    let (exact_owner, exact_lowering, exact_bytes) =
        source_free_owner_with_relative_byte_budget(exact_bytes, 0);
    let exact_candidate = prepare_source_free_successor(&exact_owner, &exact_lowering).unwrap();
    assert_eq!(
        exact_owner
            .conditional_lifecycle_probe()
            .bridge_conditional_retention()
            .unwrap()
            .retained_bytes(),
        exact_bytes,
    );
    drop(exact_candidate);
    assert_eq!(
        exact_owner
            .conditional_lifecycle_probe()
            .bridge_conditional_retention()
            .unwrap()
            .retained_bytes(),
        0,
    );

    let (short_owner, short_lowering, _) =
        source_free_owner_with_relative_byte_budget(exact_bytes, 1);
    let before_generation = short_lowering.signal_definition_generation();
    let denial = match prepare_source_free_successor(&short_owner, &short_lowering) {
        Ok(_) => panic!("one-byte-short successor retention must be denied"),
        Err(denial) => denial,
    };
    assert_eq!(
        denial.kind(),
        crate::facade::BridgeConditionalDenialKind::ConditionalRetentionCapacity,
    );
    assert_eq!(
        short_lowering.signal_definition_generation(),
        before_generation
    );
    let retained = short_owner
        .conditional_lifecycle_probe()
        .bridge_conditional_retention()
        .unwrap();
    assert_eq!(retained.retained_definition_candidates(), 0);
    assert_eq!(retained.retained_bytes(), 0);
}
