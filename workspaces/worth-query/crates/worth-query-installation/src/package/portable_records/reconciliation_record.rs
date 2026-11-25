//! Authority-free retained reconciliation-procedure carriage.

use worth_query_declaration::archive_decode_authority::ArchiveDecodeAuthority;
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryPortableInstalledReconciliationProcedureRecord {
    procedure_slot: String,
}

impl WorthQueryPortableInstalledReconciliationProcedureRecord {
    pub fn from_untrusted_procedure_slot(
        _: ArchiveDecodeAuthority,
        procedure_slot: String,
    ) -> Self {
        Self { procedure_slot }
    }

    pub(crate) fn new(procedure_slot: String) -> Self {
        Self { procedure_slot }
    }

    pub fn procedure_slot(&self) -> &str {
        &self.procedure_slot
    }
}
