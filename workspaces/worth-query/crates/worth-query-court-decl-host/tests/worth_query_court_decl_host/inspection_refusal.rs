//! An actual Host inspection outcome preserves the installed owner's exact cause.
use worth_query_decl::facade::{application_query::*, application_schema::*};
use worth_query_host::facade::{
    application_entry::WorthQueryApplicationRequestExt, causal_inspection::*,
};
use worth_query_testkit_document_retention::facade::{
    cell_history::{
        host,
        query::{LiveCells, ReadBinding as Base},
        schema::*,
    },
    document_retention_model::operator_identity::{authenticate_operator, request_scope},
};

struct Uninstalled;
impl ApplicationQueryBinding<Schema> for Uninstalled {
    type Input = <Base as ApplicationQueryBinding<Schema>>::Input;
    type InputBinding = <Base as ApplicationQueryBinding<Schema>>::InputBinding;
    type Query = <Base as ApplicationQueryBinding<Schema>>::Query;
    type ParameterBinding = <Base as ApplicationQueryBinding<Schema>>::ParameterBinding;
    type ResultBinding = <Base as ApplicationQueryBinding<Schema>>::ResultBinding;
    type ScopeBinding = <Base as ApplicationQueryBinding<Schema>>::ScopeBinding;
    type PrincipalBinding = <Base as ApplicationQueryBinding<Schema>>::PrincipalBinding;
    type Mapping = <Base as ApplicationQueryBinding<Schema>>::Mapping;
    type Principal = <Base as ApplicationQueryBinding<Schema>>::Principal;
    type PrincipalIdentity = <Base as ApplicationQueryBinding<Schema>>::PrincipalIdentity;
    type PrincipalIdentityBinding =
        <Base as ApplicationQueryBinding<Schema>>::PrincipalIdentityBinding;
    const IDENTITY: &'static str = "worth.query.test.host-uninstalled-inspection.v1";
    const LIMITS: ApplicationQueryBindingLimits = Base::LIMITS;
    fn scope_field() -> ApplicationFieldRef<
        Schema,
        Feature,
        FeatureFacts,
        FeatureKey,
        u64,
        ReadOnly,
        EqualityPredicate,
        NoApplicationUnit,
    > {
        Base::scope_field()
    }
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        Schema,
        PrincipalBinding,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        Base::principal_binding()
    }
}

#[test]
fn host_inspection_reads_exact_original_installation_cause() {
    let application = host::install();
    let scope = request_scope();
    let external = authenticate_operator(application.installed_schema(), &scope);
    let runtime = application.runtime();
    let request = runtime.request(&external, &scope);
    let result = request.query(LiveCells { feature: 1 }).execute().unwrap();
    let mut keys = result.rows().iter().map(|row| row.key).collect::<Vec<_>>();
    keys.sort_unstable();
    assert_eq!(keys, [10, 11]);
    let original = result
        .receipt()
        .inspect()
        .observation()
        .expect("genuine bound query observation");
    let denial = match request
        .inspect::<Uninstalled>(original)
        .why_observed()
        .reference_only()
        .execute()
    {
        Ok(_) => panic!("the real Host lookup must refuse an uninstalled binding"),
        Err(cause) => cause,
    };
    let Some(WorthQueryApplicationInspectionOwnerRefusal::Installation(cause)) =
        denial.owner_refusal()
    else {
        panic!("Host must preserve the original installation cause");
    };
    assert_eq!(
        cause.kind(),
        WorthQueryApplicationQueryInstallationDenialKind::BindingQueryNotInstalled
    );
    assert_eq!(cause.subject(), Uninstalled::IDENTITY);
}
