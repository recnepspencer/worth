use worth_store_physical_format::PersistedRecordIdentity;

/// Exact selected destination declaration supplied by the Store ingest owner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime) struct PreparedReuseDeclarationBasis {
    pub(in crate::physical_runtime) record: PersistedRecordIdentity,
    pub(in crate::physical_runtime) digest: [u8; 32],
}
