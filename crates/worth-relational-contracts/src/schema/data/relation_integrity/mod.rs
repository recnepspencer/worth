pub mod aggregate;
pub mod contract_id;
pub mod declarations;
pub mod lowered_plan;
pub mod plan_catalog;
pub mod plan_revision;

pub use aggregate::{
    AggregateIdentityContractDeclaration, AggregateIdentitySource, AggregateIdentityStorage,
    AggregateProjectedMemberStorage,
};
pub use contract_id::ContractId;
pub use declarations::{
    CardinalityContractDeclaration, EndpointDeletionIntegrityDeclaration,
    EndpointDeletionIntegrityMode, EndpointKindContractDeclaration, MinimumCardinalityEnforcement,
    PairMinimumSemantics, RelationIntegrityDeclarations, SymmetryContractDeclaration, SymmetryMode,
    UniquenessContractDeclaration, UniquenessScope,
};
pub use lowered_plan::{
    LoweredCardinalityMaximumContract, LoweredCardinalityMinimumContract,
    LoweredEndpointDeletionIntegrityContract, LoweredEndpointKindContract,
    LoweredRelationIntegrityPlan, LoweredSymmetryContract, LoweredUniquenessContract,
};
pub use plan_catalog::RelationIntegrityPlanCatalog;
pub use plan_revision::RelationIntegrityPlanRevision;
pub use plan_revision::{
    derive_relation_integrity_plan_revision,
    derive_relation_integrity_plan_revision_for_compilation,
};

pub mod lowering_backing;
pub mod retained_backing;
