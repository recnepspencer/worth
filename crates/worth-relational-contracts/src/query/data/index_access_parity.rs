use serde::{Deserialize, Serialize};

use crate::indexes::data::definition::DerivedIndexGenerationId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IndexParityMode {
    ProductionAdmissibility,
    SampledParity,
    CertificationParity,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IndexQueryRejectionClass {
    MissingGeneration,
    UnsupportedVersion,
    UnsupportedBranch,
    CorruptIndexEntries,
    UnsupportedScope,
    UnsupportedOrderingContract,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum QueryAccessPath {
    AuthoritativeStorage,
    DerivedIndexGeneration {
        generation_id: DerivedIndexGenerationId,
    },
    DerivedIndexRejectedStorageRead {
        rejection: IndexQueryRejectionClass,
    },
}
