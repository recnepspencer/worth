use worth_store_aspect_native::StorePhysicalBoundaryWitness;

use crate::physical_runtime::work::{
    PhysicalSignalAspectDeclaration, PhysicalWorkSemanticBasis, PhysicalWorkSignalFamily,
};

use super::super::{contract, partitioned_dependency_and_output_declaration, patch_fact};

pub(super) const ROOT_PUBLICATION_ASPECT_KEY: &str =
    "store.physical.durability.root-publication-basis";

pub(in crate::physical_runtime::record_serving::work_semantics) struct InstalledRootPublicationSemantics
{
    pub(in crate::physical_runtime::record_serving::work_semantics) basis:
        PhysicalWorkSemanticBasis,
    pub(in crate::physical_runtime::record_serving::work_semantics) declaration:
        PhysicalSignalAspectDeclaration,
}

pub(in crate::physical_runtime::record_serving::work_semantics) fn install(
    witness: StorePhysicalBoundaryWitness,
    partition: String,
) -> InstalledRootPublicationSemantics {
    let (contract, identity, admission) = contract(ROOT_PUBLICATION_ASPECT_KEY, 1_312, witness);
    let basis = PhysicalWorkSemanticBasis::mutation(
        patch_fact(
            &contract,
            identity,
            witness,
            "root-publication-work-admitted",
        ),
        admission.clone(),
    )
    .expect("root publication patch and contract are constructed together");
    InstalledRootPublicationSemantics {
        basis,
        declaration: partitioned_dependency_and_output_declaration(
            admission,
            PhysicalWorkSignalFamily::RootPublication,
            partition,
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::physical_runtime::{PhysicalSignalAspectRole, PhysicalWorkSemanticPosture};

    #[test]
    fn root_publication_is_a_dedicated_mutation_dependency_and_output() {
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
            ROOT_PUBLICATION_ASPECT_KEY
        );
        assert_eq!(
            installed.declaration.role(),
            PhysicalSignalAspectRole::DependencyAndOutput
        );
        assert_eq!(
            installed.declaration.families(),
            crate::physical_runtime::PhysicalWorkSignalFamilySet::only(
                PhysicalWorkSignalFamily::RootPublication,
            )
        );
        assert_eq!(
            installed.declaration.partition().unwrap().path().segments()[0],
            partition
        );
    }
}
