use worth_signal::facade::PartitionSubscription;
use worth_store_aspect_native::{
    StoreAspectAuthorityInput, StoreAspectBoundaryFact, StorePhysicalBoundaryWitness,
};

use crate::physical_runtime::{
    work::{
        PhysicalSignalAspectDeclaration, PhysicalSignalAspectRole, PhysicalWorkSemanticBasis,
        PhysicalWorkSignalFamily, PhysicalWorkSignalFamilySet,
    },
    PhysicalDurabilityPolicyIdentity,
};

use super::super::{projection_contract, validated_value};

const DURABILITY_POLICY_ASPECT_KEY: &str = "store.physical.durability.policy-binding-basis";

pub(in crate::physical_runtime::record_serving::work_semantics) struct InstalledDurabilityPolicySemantics
{
    pub(in crate::physical_runtime::record_serving::work_semantics) basis:
        PhysicalWorkSemanticBasis,
    pub(in crate::physical_runtime::record_serving::work_semantics) declaration:
        PhysicalSignalAspectDeclaration,
}

pub(in crate::physical_runtime::record_serving::work_semantics) fn install(
    witness: StorePhysicalBoundaryWitness,
    policy: PhysicalDurabilityPolicyIdentity,
) -> InstalledDurabilityPolicySemantics {
    install_for_partition(witness, policy_partition(policy))
}

fn install_for_partition(
    witness: StorePhysicalBoundaryWitness,
    policy_partition: String,
) -> InstalledDurabilityPolicySemantics {
    let (contract, identity, admission) =
        projection_contract(DURABILITY_POLICY_ASPECT_KEY, 1_307, witness);
    let value = validated_value(&contract, policy_partition.clone());
    let state = match worth_foundational::aspects()
        .authoritative_state()
        .admit([value])
    {
        worth_proof::TransitionOutcome::Success(state) => state,
        outcome => panic!("built-in durability policy state must admit: {outcome:?}"),
    };
    let fact = StoreAspectBoundaryFact::from_admitted_state(
        identity,
        StoreAspectAuthorityInput::new(state, witness),
    )
    .expect("durability policy state contains exactly its declared identity");
    let basis = PhysicalWorkSemanticBasis::projection(fact, admission.clone())
        .expect("durability policy fact and projection contract are constructed together");
    let families = PhysicalWorkSignalFamilySet::only(PhysicalWorkSignalFamily::WalAppend)
        .with(PhysicalWorkSignalFamily::DurabilityBarrier)
        .with(PhysicalWorkSignalFamily::CheckpointCapture)
        .with(PhysicalWorkSignalFamily::RootPublication);
    let families = families.with(PhysicalWorkSignalFamily::WalReclamation);
    let declaration =
        PhysicalSignalAspectDeclaration::new(admission, PhysicalSignalAspectRole::Dependency)
            .for_families(families)
            .with_partition(PartitionSubscription::whole_partition(policy_partition));
    InstalledDurabilityPolicySemantics { basis, declaration }
}

fn policy_partition(policy: PhysicalDurabilityPolicyIdentity) -> String {
    let digest = policy
        .bytes()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    format!("physical-durability-policy/{digest}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::physical_runtime::PhysicalWorkSemanticPosture;

    #[test]
    fn policy_binding_is_projection_only_dependency_for_exact_c7_families() {
        let partition = format!("physical-durability-policy/{}", "07".repeat(32));
        let installed = install_for_partition(
            super::super::super::security_admission::physical_witness(),
            partition.clone(),
        );
        assert_eq!(
            installed.basis.posture(),
            PhysicalWorkSemanticPosture::Projection
        );
        assert!(installed.basis.projection_fact().is_some());
        assert!(installed.basis.mutation_patch().is_none());
        assert_eq!(
            installed.basis.aspect_identity().aspect_key().as_str(),
            DURABILITY_POLICY_ASPECT_KEY
        );
        assert_eq!(
            installed.declaration.role(),
            PhysicalSignalAspectRole::Dependency
        );
        assert!(installed.declaration.contract().projection_mask().is_some());
        assert!(installed.declaration.contract().mutation_mask().is_none());
        let families = installed.declaration.families();
        for family in [
            PhysicalWorkSignalFamily::WalAppend,
            PhysicalWorkSignalFamily::DurabilityBarrier,
            PhysicalWorkSignalFamily::CheckpointCapture,
            PhysicalWorkSignalFamily::RootPublication,
            PhysicalWorkSignalFamily::WalReclamation,
        ] {
            assert!(families.contains(family));
        }
        for family in [
            PhysicalWorkSignalFamily::ReadFault,
            PhysicalWorkSignalFamily::ExactWriteback,
            PhysicalWorkSignalFamily::Publication,
            PhysicalWorkSignalFamily::Lifecycle,
        ] {
            assert!(!families.contains(family));
        }
        assert_eq!(
            installed.declaration.partition().unwrap().path().segments()[0],
            partition
        );
    }
}
