//! Closed projection shapes delegate real typed installation lookup.
use worth_query_declaration::facade::{
    application_program::{
        ApplicationNoProjection, ApplicationOutputProjection, ApplicationQueryBatchProjection,
        ApplicationQueryProjection,
    },
    application_query::ApplicationQueryBinding,
    application_schema::ApplicationSchema,
};
use worth_query_installation::facade::{
    WorthQueryApplicationQueryInstallationDenial, WorthQueryInstalledApplicationSchema,
};
mod sealed {
    pub trait Installation {}
}
impl sealed::Installation for ApplicationNoProjection {}
impl<Q> sealed::Installation for ApplicationQueryProjection<Q> {}
impl<Q, const N: usize> sealed::Installation for ApplicationQueryBatchProjection<Q, N> {}
/// Sealed output projection construction checks its actual native installed query, if present.
pub trait WorthQueryOutputProjectionInstallation<S: ApplicationSchema>:
    ApplicationOutputProjection<S> + sealed::Installation
{
    fn require_installed(
        schema: &WorthQueryInstalledApplicationSchema<S>,
    ) -> Result<(), WorthQueryApplicationQueryInstallationDenial>;
}
impl<S: ApplicationSchema, Q: ApplicationQueryBinding<S> + 'static, const N: usize>
    WorthQueryOutputProjectionInstallation<S> for ApplicationQueryBatchProjection<Q, N>
{
    fn require_installed(
        schema: &WorthQueryInstalledApplicationSchema<S>,
    ) -> Result<(), WorthQueryApplicationQueryInstallationDenial> {
        schema.installed_query_binding::<Q>().map(|_| ())
    }
}
impl<S: ApplicationSchema> WorthQueryOutputProjectionInstallation<S> for ApplicationNoProjection {
    fn require_installed(
        _: &WorthQueryInstalledApplicationSchema<S>,
    ) -> Result<(), WorthQueryApplicationQueryInstallationDenial> {
        Ok(())
    }
}
impl<S: ApplicationSchema, Q: ApplicationQueryBinding<S> + 'static>
    WorthQueryOutputProjectionInstallation<S> for ApplicationQueryProjection<Q>
{
    fn require_installed(
        schema: &WorthQueryInstalledApplicationSchema<S>,
    ) -> Result<(), WorthQueryApplicationQueryInstallationDenial> {
        schema.installed_query_binding::<Q>().map(|_| ())
    }
}
