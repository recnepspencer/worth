use std::num::NonZeroUsize;

use worth_query_host::facade::{
    admission::authenticated_principal::WorthQueryRequestScope,
    declaration::application_query::ApplicationQueryParameterSet, primary_graph,
    product::WorthQueryProductBranch,
};

use super::adapters::block_on;
use super::application::{admit_identity_adapter, ExampleApplication};
use super::schema::*;

pub fn principal(
    world: &ExampleApplication,
    request: &WorthQueryRequestScope,
) -> primary_graph::WorthQueryAuthenticatedPrincipal<TemporalHostSchema, Principal, u64> {
    let schema = world.runtime.installed_schema();
    let binding = schema
        .principal_binding(TemporalPrincipalBinding::reference())
        .expect("the installed schema must expose its principal binding");
    let authentication = admit_identity_adapter(schema);
    let external = block_on(authentication.authenticate((), request))
        .expect("the example identity must authenticate");
    world
        .runtime
        .on_branch(world.runtime.current_world())
        .select()
        .expect("the current product must be selectable")
        .resolve_authenticated_principal(
            &binding,
            &external,
            request,
            primary_graph::WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .expect("the authenticated principal must resolve")
}

pub fn controls(
    request: &WorthQueryRequestScope,
) -> primary_graph::WorthQueryProductQueryControls<'_> {
    primary_graph::WorthQueryProductQueryControls::new(
        NonZeroUsize::new(8).expect("nonzero row budget"),
        NonZeroUsize::new(64).expect("nonzero byte budget"),
        request,
    )
}

pub fn read_input<'runtime>(
    world: &'runtime ExampleApplication,
    branch: WorthQueryProductBranch,
    principal: &'runtime primary_graph::WorthQueryAuthenticatedPrincipal<
        TemporalHostSchema,
        Principal,
        u64,
    >,
    request: &'runtime WorthQueryRequestScope,
) -> String {
    let selected = world
        .runtime
        .on_branch(branch)
        .select()
        .expect("the product branch must be selectable");
    read_selected(world, selected, principal, request)
}

pub fn read_selected<'runtime>(
    world: &'runtime ExampleApplication,
    selected: primary_graph::WorthQuerySelectedProductOperation<'runtime, TemporalHostSchema>,
    principal: &'runtime primary_graph::WorthQueryAuthenticatedPrincipal<
        TemporalHostSchema,
        Principal,
        u64,
    >,
    request: &'runtime WorthQueryRequestScope,
) -> String {
    let scope = selected
        .resolve_entity(
            IntentIdentityField::reference(),
            "intent-1".to_owned(),
            request,
            primary_graph::WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .expect("the product branch must resolve the intent");
    let access = primary_graph::WorthQueryApplicationQueryAccessContext::new(principal, &scope);
    let query = world
        .runtime
        .installed_schema()
        .certification_query(TemporalIntentQuery::reference())
        .expect("the application query must be installed");
    let admitted = selected
        .admit_application_query(
            &query,
            &access,
            ApplicationQueryParameterSet::new(),
            controls(request),
        )
        .expect("the exact product read must be admitted");
    let result = world
        .runtime
        .execute_application_query_one_shot(admitted)
        .expect("the exact product read must execute");
    result.rows()[0].input.clone()
}

pub fn product_identity(
    receipt: &primary_graph::WorthQueryApplicationQueryAccessReceipt,
) -> &primary_graph::WorthQueryProductBranchReadIdentity {
    let primary_graph::WorthQueryApplicationBasisSelectionIdentity::Product(identity) =
        receipt.basis_identity().selection()
    else {
        panic!("the public product read must retain product identity")
    };
    identity
}
