//! Register the real installed action and retain its ingress worker before serving.
use super::{
    contract::WorthServerQuerySourceAdapter, protocol, transport, WorthServerSourceCustodyLimits,
    WorthServerSourceCustodyMount,
};
use crate::{
    WorthServerProductApplicationAdapterRegistration, WorthServerProductOperationBasisKind,
    WorthServerProductOperationDeclaration, WorthServerProductOperationErrorMaps,
    WorthServerProductOperationSupportSnapshot, WorthServerProductRetryOwner,
};
use std::sync::Arc;
/// Start one bounded transport worker. Query retains all mutation authority.
pub fn mount_query_source<C: WorthServerQuerySourceAdapter>(
    adapter: C,
    limits: WorthServerSourceCustodyLimits,
) -> std::io::Result<WorthServerSourceCustodyMount> {
    let program = adapter.program().clone();
    let continuation = transport::start(Arc::new(adapter), limits)?;
    let declaration = WorthServerProductOperationDeclaration::product_mutation(
        super::contract::operation::<C>(),
        C::INPUT,
        protocol::result_contract(C::RESULT),
        WorthServerProductOperationBasisKind::PrimaryGraphApplication,
        WorthServerProductOperationSupportSnapshot::production_admitted(C::NAMESPACE),
        C::NAMESPACE,
    )
    .with_program_application(program)
    .with_retry_owner(WorthServerProductRetryOwner::ApplicationCustody)
    .with_payload_validator(Arc::new(protocol::CommandInput::<C>::default()))
    .with_error_map(WorthServerProductOperationErrorMaps::passthrough());
    Ok(WorthServerSourceCustodyMount {
        registration: WorthServerProductApplicationAdapterRegistration::new(
            super::contract::operation::<C>(),
            Arc::new(continuation.clone()),
        )
        .with_operation(declaration),
        continuation,
    })
}
