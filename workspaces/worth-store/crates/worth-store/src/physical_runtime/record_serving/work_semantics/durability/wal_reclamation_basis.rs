use worth_store_aspect_native::StorePhysicalBoundaryWitness;

use crate::physical_runtime::work::{
    PhysicalSignalAspectDeclaration, PhysicalWorkSemanticBasis, PhysicalWorkSignalFamily,
};

use super::super::{contract, partitioned_dependency_and_output_declaration, patch_fact};

const WAL_RECLAMATION_ASPECT_KEY: &str = "store.physical.durability.wal-reclamation-basis";

pub(in crate::physical_runtime::record_serving::work_semantics) struct InstalledWalReclamationSemantics
{
    pub(in crate::physical_runtime::record_serving::work_semantics) basis:
        PhysicalWorkSemanticBasis,
    pub(in crate::physical_runtime::record_serving::work_semantics) declaration:
        PhysicalSignalAspectDeclaration,
}

pub(in crate::physical_runtime::record_serving::work_semantics) fn install(
    witness: StorePhysicalBoundaryWitness,
    partition: String,
) -> InstalledWalReclamationSemantics {
    let (contract, identity, admission) = contract(WAL_RECLAMATION_ASPECT_KEY, 1_311, witness);
    let basis = PhysicalWorkSemanticBasis::mutation(
        patch_fact(
            &contract,
            identity,
            witness,
            "wal-reclamation-work-admitted",
        ),
        admission.clone(),
    )
    .expect("WAL reclamation patch and contract are constructed together");
    InstalledWalReclamationSemantics {
        basis,
        declaration: partitioned_dependency_and_output_declaration(
            admission,
            PhysicalWorkSignalFamily::WalReclamation,
            partition,
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::physical_runtime::{PhysicalSignalAspectRole, PhysicalWorkSemanticPosture};

    #[test]
    fn wal_reclamation_is_a_dedicated_mutation_dependency_and_output() {
        let partition = "physical-durability-runtime/store/runtime".to_owned();
        let installed = install(
            super::super::super::security_admission::physical_witness(),
            partition.clone(),
        );
        assert_eq!(
            installed.basis.posture(),
            PhysicalWorkSemanticPosture::Mutation
        );
        assert_eq!(
            installed.basis.aspect_identity().aspect_key().as_str(),
            WAL_RECLAMATION_ASPECT_KEY
        );
        assert_eq!(
            installed.declaration.role(),
            PhysicalSignalAspectRole::DependencyAndOutput
        );
        assert_eq!(
            installed.declaration.partition().unwrap().path().segments()[0],
            partition
        );
    }
}
