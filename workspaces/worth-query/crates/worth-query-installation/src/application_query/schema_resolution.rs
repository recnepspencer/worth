use worth_query_declaration::facade::{
    application_query::ApplicationQueryReference, application_schema::ApplicationSchema,
};

use crate::application_schema::WorthQueryInstalledApplicationSchema;

use super::{
    binding::ApplicationQueryBindingKey, WorthQueryApplicationQueryInstallationDenial,
    WorthQueryApplicationQueryInstallationDenialKind, WorthQueryInstalledApplicationQuery,
};

impl<Schema> WorthQueryInstalledApplicationSchema<Schema>
where
    Schema: ApplicationSchema,
{
    /// Resolves raw installed query authority for lower-level certification
    /// fixtures, including intentionally malformed queries that cannot form a
    /// valid application binding.
    pub fn certification_query<Query, Parameters, QueryResult, Scope>(
        &self,
        reference: ApplicationQueryReference<Schema, Query, Parameters, QueryResult, Scope>,
    ) -> Result<
        WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>,
        WorthQueryApplicationQueryInstallationDenial,
    > {
        let key = ApplicationQueryBindingKey::from_reference(&reference);
        let compiled = self
            .query_catalog
            .get_query(&key)
            .or_else(|| self.query_catalog.get_query_by_name(reference.name()))
            .ok_or_else(|| {
                WorthQueryApplicationQueryInstallationDenial::new(
                    WorthQueryApplicationQueryInstallationDenialKind::QueryNotInstalled,
                    reference.name(),
                )
            })?;
        if !compiled.matches_reference(reference) {
            return Err(WorthQueryApplicationQueryInstallationDenial::new(
                WorthQueryApplicationQueryInstallationDenialKind::QueryMeaningChanged,
                reference.name(),
            ));
        }
        Ok(WorthQueryInstalledApplicationQuery::from_compiled(compiled))
    }
}
