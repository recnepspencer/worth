use super::progression::AspectContractShapeStep;
use crate::aspects::{
    AbsenceLaw, AspectContract, AspectEquivalenceBasis, AspectEvolutionPolicy,
    AspectFrontDoorConstructionDenial, AspectMaskContract, AspectShape, OpaqueAspectType,
    ReferenceAspectType, StructAspectShape,
};
use crate::values::ScalarAspectType;

impl AspectContractShapeStep {
    pub fn scalar(self, scalar: ScalarAspectType) -> AspectContract {
        AspectContract::scalar(self.key, self.identity, self.revision, scalar)
    }

    pub fn scalar_with(
        self,
        scalar: ScalarAspectType,
        masks: AspectMaskContract,
        absence: AbsenceLaw,
        equivalence: AspectEquivalenceBasis,
        evolution: AspectEvolutionPolicy,
    ) -> AspectContract {
        self.finish_custom_contract(
            AspectShape::Scalar(scalar),
            masks,
            absence,
            equivalence,
            evolution,
        )
    }

    pub fn struct_aspect(self, shape: StructAspectShape) -> AspectContract {
        AspectContract::struct_aspect(self.key, self.identity, self.revision, shape)
    }

    pub fn struct_with(
        self,
        shape: StructAspectShape,
        masks: AspectMaskContract,
        absence: AbsenceLaw,
        equivalence: AspectEquivalenceBasis,
        evolution: AspectEvolutionPolicy,
    ) -> AspectContract {
        self.finish_custom_contract(
            AspectShape::Struct(shape),
            masks,
            absence,
            equivalence,
            evolution,
        )
    }

    pub fn reference_entity(self) -> AspectContract {
        AspectContract::reference_entity(self.key, self.identity, self.revision)
    }

    pub fn opaque_token(self) -> AspectContract {
        AspectContract::opaque_token(self.key, self.identity, self.revision)
    }

    pub fn opaque_with(
        self,
        opaque: OpaqueAspectType,
        masks: AspectMaskContract,
        absence: AbsenceLaw,
        equivalence: AspectEquivalenceBasis,
        evolution: AspectEvolutionPolicy,
    ) -> Result<AspectContract, AspectFrontDoorConstructionDenial> {
        if !mask_contract_is_opaque_diagnostic_only(&masks) {
            return Err(AspectFrontDoorConstructionDenial::OpaqueMaskContractMustBeDiagnosticOnly);
        }

        Ok(self.finish_custom_contract(
            AspectShape::Opaque(opaque),
            masks,
            absence,
            equivalence,
            evolution,
        ))
    }

    pub fn reference_with(
        self,
        reference: ReferenceAspectType,
        masks: AspectMaskContract,
        absence: AbsenceLaw,
        equivalence: AspectEquivalenceBasis,
        evolution: AspectEvolutionPolicy,
    ) -> AspectContract {
        self.finish_custom_contract(
            AspectShape::Reference(reference),
            masks,
            absence,
            equivalence,
            evolution,
        )
    }

    fn finish_custom_contract(
        self,
        shape: AspectShape,
        masks: AspectMaskContract,
        absence: AbsenceLaw,
        equivalence: AspectEquivalenceBasis,
        evolution: AspectEvolutionPolicy,
    ) -> AspectContract {
        AspectContract::new(
            self.key,
            self.identity,
            self.revision,
            shape,
            masks,
            absence,
            equivalence,
            evolution,
        )
    }
}

fn mask_contract_is_opaque_diagnostic_only(masks: &AspectMaskContract) -> bool {
    !masks.projection_allowed() && !masks.mutation_allowed() && masks.diagnostic_allowed()
}
