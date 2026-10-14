//! Closed projection shapes delegate real typed installation lookup.
use worth_query_declaration::facade::{
    application_program::{
        ApplicationFixedProjection, ApplicationNoProjection, ApplicationQueryProjection,
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
/// Sealed fixed projection construction checks its actual native installed query, if present.
pub trait WorthQueryFixedProjectionInstallation<S: ApplicationSchema>:
    ApplicationFixedProjection<S> + sealed::Installation
{
    fn require_installed(
        schema: &WorthQueryInstalledApplicationSchema<S>,
    ) -> Result<(), WorthQueryApplicationQueryInstallationDenial>;
}
impl<S: ApplicationSchema> WorthQueryFixedProjectionInstallation<S> for ApplicationNoProjection {
    fn require_installed(
        _: &WorthQueryInstalledApplicationSchema<S>,
    ) -> Result<(), WorthQueryApplicationQueryInstallationDenial> {
        Ok(())
    }
}
impl<S: ApplicationSchema, Q: ApplicationQueryBinding<S> + 'static>
    WorthQueryFixedProjectionInstallation<S> for ApplicationQueryProjection<Q>
{
    fn require_installed(
        schema: &WorthQueryInstalledApplicationSchema<S>,
    ) -> Result<(), WorthQueryApplicationQueryInstallationDenial> {
        schema.installed_query_binding::<Q>().map(|_| ())
    }
}
