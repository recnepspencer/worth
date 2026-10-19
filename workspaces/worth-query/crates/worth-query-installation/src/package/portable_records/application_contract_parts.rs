//! Decoded, authority-free parts for retained application contract records.

use std::collections::BTreeSet;

use worth_foundational::facade::{AspectBinding, AspectContract, AspectKey, FieldKey};
use worth_query_declaration::facade::application_schema::{
    ApplicationExternalEffectProtocol, WorthQueryExternalEffectCorrelationFamily,
};
use worth_query_declaration::facade::portable_identity::WorthQueryPortableTypeIdentity;

use super::{
    WorthQueryPortableExternalEffectContractRecord,
    WorthQueryPortableInstalledReconciliationProcedureRecord,
    WorthQueryPortableOperationGraphReadScope, WorthQueryPortableOperationTouchScope,
};

pub struct WorthQueryPortableNativeAspectContractParts {
    pub schema: String,
    pub entity: String,
    pub aspect: AspectKey,
    pub contract: AspectContract,
    pub fields: BTreeSet<FieldKey>,
    pub binding: AspectBinding,
}

pub struct WorthQueryPortableExternalEffectContractParts {
    pub correlation_family: WorthQueryExternalEffectCorrelationFamily,
    pub effect: String,
    pub payload_type: WorthQueryPortableTypeIdentity,
    pub protocol: ApplicationExternalEffectProtocol,
    pub maximum_payload_bytes: u64,
}

pub struct WorthQueryPortableApplicationOperationContractParts {
    pub schema: String,
    pub operation: String,
    pub input_type: WorthQueryPortableTypeIdentity,
    pub graph_reads: Vec<WorthQueryPortableOperationGraphReadScope>,
    pub touches: Vec<WorthQueryPortableOperationTouchScope>,
    pub emissions: Vec<String>,
    pub external_effect: Option<WorthQueryPortableExternalEffectContractRecord>,
    pub reconciliation: Option<WorthQueryPortableInstalledReconciliationProcedureRecord>,
    pub body: Option<
        worth_query_declaration::facade::application_schema::ApplicationOperationBodyProgram,
    >,
}
