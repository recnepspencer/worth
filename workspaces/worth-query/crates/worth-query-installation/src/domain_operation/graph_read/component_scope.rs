//! Closed member visibility meaning retained by operation installation.
use crate::application_query::WorthQueryInstalledRootPath;
use std::sync::Arc;
use worth_query_declaration::facade::application_schema::{
    ApplicationOperationComponentReadMeaning, ApplicationSchemaBindingIdentity,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryOperationComponentReadScope {
    schema: ApplicationSchemaBindingIdentity,
    meaning: Arc<ApplicationOperationComponentReadMeaning>,
    paths: Arc<[WorthQueryInstalledRootPath]>,
    meaning_identity: [u8; 32],
}

impl WorthQueryOperationComponentReadScope {
    pub(crate) fn new(
        schema: ApplicationSchemaBindingIdentity,
        meaning: ApplicationOperationComponentReadMeaning,
    ) -> Self {
        use sha2::{Digest, Sha256};
        let meaning_identity = Sha256::digest(meaning.canonical_record().as_bytes()).into();
        let paths = meaning
            .paths()
            .iter()
            .map(WorthQueryInstalledRootPath::from_meaning)
            .collect();
        Self {
            schema,
            meaning: Arc::new(meaning),
            paths,
            meaning_identity,
        }
    }

    pub fn schema(&self) -> &ApplicationSchemaBindingIdentity {
        &self.schema
    }

    pub fn meaning(&self) -> &ApplicationOperationComponentReadMeaning {
        &self.meaning
    }

    /// Canonical visibility meaning, independent of Native component naming.
    pub const fn meaning_identity(&self) -> [u8; 32] {
        self.meaning_identity
    }

    pub fn paths(&self) -> &[WorthQueryInstalledRootPath] {
        &self.paths
    }
}
