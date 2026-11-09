//! Successor-owned native contribution for the additive B application.
use super::schema::{
    Account, AccountIdentity, HomeRecords, IdentityBinding, LaterRecords, SuccessorSchema,
};
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
pub struct ExtendedSurface;
impl ApplicationSchemaContribution<SuccessorSchema> for ExtendedSurface {
    const IDENTITY: ApplicationSchemaContributionIdentity =
        ApplicationSchemaContributionIdentity::new("worth.query.host.home.native-surface.v1");
    type Members = (
        HomeRecords,
        LaterRecords,
        IdentityBinding,
        SetBalance,
        SetBalanceBinding,
        balance_query::AccountQuery,
        balance_query::AccountReadBinding,
    );
}
impl Registration<SuccessorSchema> for SetBalance {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<SuccessorSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<SuccessorSchema>,
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
impl Registration<SuccessorSchema> for SetBalanceBinding {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<SuccessorSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<SuccessorSchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(builder.application_mutation_binding::<Self>())
    }
}
impl Registration<SuccessorSchema> for balance_query::AccountQuery {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<SuccessorSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<SuccessorSchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(builder.application_query(balance_query::definition()))
    }
}
impl Registration<SuccessorSchema> for balance_query::AccountReadBinding {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<SuccessorSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<SuccessorSchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(builder.application_query_binding::<Self>())
    }
}
impl WorthQueryApplicationContribution<SuccessorSchema> for ExtendedSurface {
    type Configuration = std::sync::Arc<std::sync::atomic::AtomicUsize>;
    fn configure(
        configuration: Self::Configuration,
        setup: &mut WorthQueryApplicationContributionSetup<'_, SuccessorSchema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        setup.handler::<SetBalanceBinding, _>(BalanceHandler(configuration))?;
        Ok(())
    }
}
