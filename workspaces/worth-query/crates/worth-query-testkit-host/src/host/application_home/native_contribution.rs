//! A Home-only authored contribution; existing identity fixtures stay unchanged.
use super::schema::{Account, AccountIdentity, HomeIdentitySchema, HomeRecords, IdentityBinding};
use super::{balance_query, native_mutation::*};
use worth_query_decl::facade::application_schema::{
    fragment::{FieldSelection, Registration},
    *,
};
use worth_query_host::facade::{
    application_contribution::{
        WorthQueryApplicationContribution, WorthQueryApplicationContributionSetup,
    },
    primary_graph::WorthQueryPrimaryGraphInstallationDenial,
};
pub struct HomeNativeSurface;
impl ApplicationSchemaContribution<HomeIdentitySchema> for HomeNativeSurface {
    const IDENTITY: ApplicationSchemaContributionIdentity =
        ApplicationSchemaContributionIdentity::new("worth.query.host.home.native-surface.v1");
    type Members = (
        HomeRecords,
        IdentityBinding,
        SetBalance,
        SetBalanceBinding,
        balance_query::AccountQuery,
        balance_query::AccountReadBinding,
    );
}
impl Registration<HomeIdentitySchema> for SetBalance {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<HomeIdentitySchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<HomeIdentitySchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(builder
            .operation(
                SetBalance::reference()
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .operation_projection_work_budget(SetBalance::reference(), 4)
            .operation_read_field(
                SetBalance::reference(),
                FieldSelection::<HomeRecords, Account, AccountIdentity, String>::at(
                    "AccountNumber",
                ),
            )
            .operation_read_field(
                SetBalance::reference(),
                FieldSelection::<HomeRecords, Account, AccountIdentity, u64>::at("Balance"),
            )
            .operation_write(
                SetBalance::reference(),
                FieldSelection::<HomeRecords, Account, AccountIdentity, u64>::at("Balance"),
            ))
    }
}
impl Registration<HomeIdentitySchema> for SetBalanceBinding {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<HomeIdentitySchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<HomeIdentitySchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(builder.application_mutation_binding::<Self>())
    }
}
impl Registration<HomeIdentitySchema> for balance_query::AccountQuery {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<HomeIdentitySchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<HomeIdentitySchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(builder.application_query(balance_query::definition()))
    }
}
impl Registration<HomeIdentitySchema> for balance_query::AccountReadBinding {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<HomeIdentitySchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<HomeIdentitySchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(builder.application_query_binding::<Self>())
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
