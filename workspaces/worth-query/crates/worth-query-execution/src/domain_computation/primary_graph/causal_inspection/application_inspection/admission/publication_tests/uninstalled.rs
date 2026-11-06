//! Real installed lookup refuses an undeclared binding before inspection projection.
use crate::domain_computation::primary_graph::{
    admitted_capture::CaptureStop,
    causal_inspection::application_inspection::{
        request::binding::InspectionBindingDenial, stop::InspectionCause,
        WorthQueryApplicationInspectionDenial, WorthQueryApplicationInspectionOwnerRefusal,
    },
    tests::fixture::{
        AccountStatus, ExternalMapping, IdentityBinding, IdentityExecutionSchema, Principal,
        TestAccountSourceBinding as Base,
    },
};
use worth_execution::{MapKernelFailure, ScopeStop};
use worth_query_declaration::facade::{application_query::*, application_schema::*};
type S = IdentityExecutionSchema;
struct Uninstalled;
impl ApplicationQueryBinding<S> for Uninstalled {
    type Input = <Base as ApplicationQueryBinding<S>>::Input;
    type InputBinding = <Base as ApplicationQueryBinding<S>>::InputBinding;
    type Query = <Base as ApplicationQueryBinding<S>>::Query;
    type ParameterBinding = <Base as ApplicationQueryBinding<S>>::ParameterBinding;
    type ResultBinding = <Base as ApplicationQueryBinding<S>>::ResultBinding;
    type ScopeBinding = <Base as ApplicationQueryBinding<S>>::ScopeBinding;
    type PrincipalBinding = <Base as ApplicationQueryBinding<S>>::PrincipalBinding;
    type Mapping = <Base as ApplicationQueryBinding<S>>::Mapping;
    type Principal = <Base as ApplicationQueryBinding<S>>::Principal;
    type PrincipalIdentity = <Base as ApplicationQueryBinding<S>>::PrincipalIdentity;
    type PrincipalIdentityBinding = <Base as ApplicationQueryBinding<S>>::PrincipalIdentityBinding;
    const IDENTITY: &'static str = "worth.query.test.uninstalled-inspection-cause.v1";
    const LIMITS: ApplicationQueryBindingLimits = Base::LIMITS;
    fn scope_field() -> AccountStatus {
        Base::scope_field()
    }
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        S,
        IdentityBinding,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        Base::principal_binding()
    }
}

pub(super) fn assert_original_installation_cause(
    installed: &worth_query_installation::facade::WorthQueryInstalledApplicationSchema<S>,
) {
    let original = installed
        .installed_query_binding::<Uninstalled>()
        .err()
        .expect("not declared by the real schema");
    let denied = WorthQueryApplicationInspectionDenial::new(
        InspectionCause::Binding(CaptureStop::Kernel(ScopeStop::Failure(
            MapKernelFailure::Domain(InspectionBindingDenial::Installation(original.clone())),
        ))),
        None,
    );
    let Some(WorthQueryApplicationInspectionOwnerRefusal::Installation(actual)) =
        denied.owner_refusal()
    else {
        panic!("installation cause must survive")
    };
    assert_eq!(actual, &original);
    assert_eq!(actual.subject(), Uninstalled::IDENTITY);
}
