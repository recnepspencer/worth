//! A Home-only authored contribution; existing identity fixtures stay unchanged.
use super::schema::{Account, AccountNumber, HomeIdentitySchema};
use super::{balance_query, native_mutation::*};
use worth_query_decl::facade::worth_query_application_contribution;
use worth_query_host::facade::{
    application_contribution::{
        WorthQueryApplicationContribution, WorthQueryApplicationContributionSetup,
    },
    primary_graph::WorthQueryPrimaryGraphInstallationDenial,
};
worth_query_application_contribution! {
    pub contribution HomeNativeSurface in HomeIdentitySchema {
        identity: "worth.query.host.home.native-surface.v1",
        members: |schema| {
            super::schema::declare(schema).field(Account::reference(), Balance::reference())
                .operation(SetBalance::reference().definition().no_external_effect().no_aftermath().finish())
                .operation_projection_work_budget(SetBalance::reference(), 4)
                .operation_read_field(SetBalance::reference(), AccountNumber::reference())
                .operation_read_field(SetBalance::reference(), Balance::reference())
                .operation_write(SetBalance::reference(), Balance::reference())
                .application_mutation_binding::<SetBalanceBinding>()
                .application_query(balance_query::definition())
                .application_query_binding::<balance_query::AccountReadBinding>()
        }
    }
}
impl WorthQueryApplicationContribution<HomeIdentitySchema> for HomeNativeSurface {
    type Configuration = std::sync::Arc<std::sync::atomic::AtomicUsize>;
    fn configure(
        configuration: Self::Configuration,
        setup: &mut WorthQueryApplicationContributionSetup<'_, HomeIdentitySchema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        setup.handler::<SetBalanceBinding, _>(BalanceHandler(configuration))?;
        Ok(())
    }
}
