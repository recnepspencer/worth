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
}

impl WorthQueryOperationComponentReadScope {
    pub(crate) fn new(
        schema: ApplicationSchemaBindingIdentity,
        meaning: ApplicationOperationComponentReadMeaning,
    ) -> Self {
        let paths = meaning
            .paths()
            .iter()
            .map(WorthQueryInstalledRootPath::from_meaning)
            .collect();
        Self {
            schema,
            meaning: Arc::new(meaning),
            paths,
        }
    }

    pub fn schema(&self) -> &ApplicationSchemaBindingIdentity {
        &self.schema
    }

    pub fn meaning(&self) -> &ApplicationOperationComponentReadMeaning {
        &self.meaning
    }

    pub fn paths(&self) -> &[WorthQueryInstalledRootPath] {
        &self.paths
    }
}
