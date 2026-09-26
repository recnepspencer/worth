use worth_query_host::facade::application_entry::{
    WorthQueryApplicationOutputDemandProgress, WorthQueryApplicationRequest,
    WorthQueryOutputDemandControls,
};
use worth_query_topology_entry::PlanarOutputDemand;

use crate::ConsumerSchema;

pub(in crate::application_invariant_acceptance::proof::application_program) fn settle_recovered<'application>(
    request: &'application WorthQueryApplicationRequest<'application, '_, '_, ConsumerSchema>,
    body_key: &str,
    controls: WorthQueryOutputDemandControls,
) -> worth_query_host::facade::application_entry::WorthQueryApplicationOutputDemandSettlement<
    <worth_query_topology_entry::PlanarReadBinding<ConsumerSchema> as worth_query_decl::facade::application_query::ApplicationQueryBinding<ConsumerSchema>>::Query,
>{
    let mut recovered = request
        .demand(PlanarOutputDemand::new(body_key))
        .controls(controls)
        .start()
        .expect("owner custody is recoverable through the public demand entry");
    crate::application_invariant_acceptance::proof::settle(|| {
        match recovered.advance(request).unwrap_or_else(|denial| {
            panic!("the recovered {body_key} required output advances: {denial:?}")
        }) {
            WorthQueryApplicationOutputDemandProgress::Pending => None,
            WorthQueryApplicationOutputDemandProgress::Settled(settled) => Some(settled),
        }
    })
}
