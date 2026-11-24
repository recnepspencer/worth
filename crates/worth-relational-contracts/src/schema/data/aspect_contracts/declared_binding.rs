use worth_foundational::facade::AspectKey;
use worth_foundational::AspectContract;

pub use worth_foundational::facade::AspectBinding;
pub use worth_foundational::facade::AuthoritativeAspectChangeKind as RelationalAspectChangeKind;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeclaredAspectContractBinding {
    pub binding: AspectBinding,
    pub contract: AspectContract,
}

impl DeclaredAspectContractBinding {
    pub fn aspect_key(&self) -> AspectKey {
        self.contract.key().clone()
    }

    pub fn foundational_key(&self) -> &worth_foundational::AspectKey {
        self.contract.key()
    }
}
