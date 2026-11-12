//! Shared ordinary registration of the authored model's Relational handlers.
use super::*;
pub(in super::super) struct RelationalContribution;
impl ApplicationSchemaContribution<Model<60>> for RelationalContribution {
    const IDENTITY: ApplicationSchemaContributionIdentity =
        <Contribution as ApplicationSchemaContribution<Model<60>>>::IDENTITY;
    type Members = <Contribution as ApplicationSchemaContribution<Model<60>>>::Members;
}
impl WorthQueryApplicationContribution<Model<60>> for RelationalContribution {
    type Configuration = ();
    fn configure(
        _: (),
        setup: &mut WorthQueryApplicationContributionSetup<'_, Model<60>>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        configure(setup)
    }
}
impl ApplicationSchemaContribution<Model<0>> for RelationalContribution {
    const IDENTITY: ApplicationSchemaContributionIdentity =
        <Contribution as ApplicationSchemaContribution<Model<0>>>::IDENTITY;
    type Members = <Contribution as ApplicationSchemaContribution<Model<0>>>::Members;
}
impl WorthQueryApplicationContribution<Model<0>> for RelationalContribution {
    type Configuration = ();
    fn configure(
        _: (),
        setup: &mut WorthQueryApplicationContributionSetup<'_, Model<0>>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        configure(setup)
    }
}
pub(super) fn configure<S: Scenario>(
    setup: &mut WorthQueryApplicationContributionSetup<'_, S>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    if matches!(S::MODE, 30 | 39 | 40 | 41 | 60) {
        setup.handler::<relation_splice::inspection::Binding<S>, _>(
            relation_splice::inspection::Handler,
        )?;
    }
    if S::MODE == 29 {
        setup.handler::<projected_memberships::inspection::Binding<S>, _>(
            projected_memberships::inspection::Handler,
        )?;
    }
    if matches!(S::MODE, 48 | 68 | 69 | 70) {
        setup.handler::<parameter_inverse::Binding<S>, _>(parameter_inverse::Handler)?;
    }
    if S::MODE == 70 {
        setup.handler::<foreign_aggregate::BirthBinding<S>, _>(
            foreign_aggregate::command::Handler,
        )?;
        setup.handler::<sheet_boundary::split_walk::ExtendBinding<S>, _>(
            sheet_boundary::split_walk::Handler,
        )?;
    }
    super::owned_action_feature::configure_create::<S>(setup)?;
    setup.handler::<lifecycle::RetireBinding<S>, _>(lifecycle_handlers::Handler)?;
    setup.handler::<lifecycle::RestoreBinding<S>, _>(lifecycle_handlers::Handler)?;
    setup.handler::<lifecycle::PlainBinding<S>, _>(lifecycle_handlers::Handler)?;
    setup.handler::<producer::Binding<S>, _>(producer::Provider)?;
    if S::COUPLED {
        setup.handler::<coupled::producer::Binding<S>, _>(coupled::producer::Provider)?;
    }
    Ok(())
}
