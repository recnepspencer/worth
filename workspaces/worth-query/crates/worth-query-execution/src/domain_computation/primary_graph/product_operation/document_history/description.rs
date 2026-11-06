//! Process-local description correspondence grants no operation authority.
use super::*;
use crate::publication_boundary::WorthQueryProgramPublicationAccess;

/// One actual process-local document incarnation, carrying no permission or lease.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryDocumentHistoryIdentity(pub(super) step::DocumentIncarnation);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryHistoryDescriptionContext {
    version: u32,
    schema: ApplicationSchemaBindingIdentity,
    document: WorthQueryDocumentHistoryIdentity,
}

impl WorthQueryHistoryDescriptionContext {
    pub const CURRENT_VERSION: u32 = 1;
    pub fn version(&self) -> u32 {
        self.version
    }
    pub fn schema_binding(&self) -> &ApplicationSchemaBindingIdentity {
        &self.schema
    }
    pub fn document(&self) -> WorthQueryDocumentHistoryIdentity {
        self.document
    }

    /// Describe an incoming format. Unsupported formats receive no interpretation.
    /// This copy conveys no READ, policy or navigation permission.
    pub fn for_version(mut self, version: u32) -> Self {
        self.version = version;
        self
    }

    fn check_schema(
        &self,
        schema: &ApplicationSchemaBindingIdentity,
    ) -> Result<(), WorthQueryHistoryCommandContextDenial> {
        use WorthQueryHistoryCommandContextDenial as Denial;
        if self.version != Self::CURRENT_VERSION {
            return Err(Denial::UnsupportedVersion {
                received: self.version,
                supported: Self::CURRENT_VERSION,
            });
        }
        if self.schema != *schema {
            return Err(Denial::ForeignSchemaBinding);
        }
        Ok(())
    }

    fn check_document(
        &self,
        document: WorthQueryDocumentHistoryIdentity,
    ) -> Result<(), WorthQueryHistoryCommandContextDenial> {
        if self.document != document {
            return Err(WorthQueryHistoryCommandContextDenial::ForeignDocument);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryHistoryCommandContextDenial {
    UnsupportedVersion { received: u32, supported: u32 },
    ForeignSchemaBinding,
    ForeignDocument,
}

impl<S: ApplicationSchema, H: ApplicationHistoryBinding<S>> WorthQueryDocumentHistory<'_, S, H> {
    pub fn history_document_identity(
        &self,
        _access: &WorthQueryProgramPublicationAccess,
    ) -> WorthQueryDocumentHistoryIdentity {
        self.selection.lock().journal.document_identity()
    }

    pub fn history_description_context(
        &self,
        access: &WorthQueryProgramPublicationAccess,
    ) -> WorthQueryHistoryDescriptionContext {
        WorthQueryHistoryDescriptionContext {
            version: WorthQueryHistoryDescriptionContext::CURRENT_VERSION,
            schema: self.binding.clone(),
            document: self.history_document_identity(access),
        }
    }

    pub fn check_history_command_context(
        &self,
        access: &WorthQueryProgramPublicationAccess,
        context: &WorthQueryHistoryDescriptionContext,
    ) -> Result<(), WorthQueryHistoryCommandContextDenial> {
        context.check_schema(&self.binding)?;
        context.check_document(self.history_document_identity(access))
    }
}
