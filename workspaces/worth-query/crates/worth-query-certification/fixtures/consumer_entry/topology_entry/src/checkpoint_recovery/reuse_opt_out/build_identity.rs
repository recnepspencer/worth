//! One real None-producer checkpoint across same and changed application builds.
use super::*;
use worth_query_consumer_values::{
    PlanarCurrentOutputExpectation, PlanarMutationDenial, PlanarOperation,
};
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationMutationOutcome as Outcome, WorthQueryApplicationOutputDemandSettlement,
    WorthQueryApplicationRequest, WorthQueryOutputSettlementPosture as Posture,
};
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;

type Application =
    application_installation::WorthQueryProgramApplicationRuntime<NoReuseSchema, NoReuseProgram>;
type Request<'a, 'p, 's> = WorthQueryApplicationRequest<'a, 'p, 's, NoReuseSchema>;

fn reopen(bytes: &[u8]) -> application_installation::WorthQueryApplicationCheckpoint {
    application_installation::WorthQueryApplicationCheckpoint::from_untrusted_bytes(bytes.to_vec())
}

fn reset_contacts(provider_observation: &crate::producer::ProviderContacts) {
    provider_observation.reset();
    crate::final_output::reset_provider_contacts();
}

fn root_demand<'a, 'p, 's>(
    request: &Request<'a, 'p, 's>,
    application: &'a Application,
) -> WorthQueryApplicationOutputDemandSettlement<PlanarQuery> {
    let mut demand = request
        .demand(PlanarOutputDemand::new("anchor-a"))
        .start_in_program::<NoReuseProgram, program::Root>(application)
        .unwrap();
    for _ in 0..64 {
        if let WorthQueryApplicationOutputDemandProgress::Settled(settled) =
            demand.advance(request).unwrap()
        {
            return settled;
        }
    }
    panic!("the real root demand must settle");
}

fn final_demand<'a, 'p, 's>(
    request: &Request<'a, 'p, 's>,
    application: &'a Application,
) -> WorthQueryApplicationOutputDemandSettlement<PlanarQuery> {
    let mut demand = request
        .demand(PlanarFinalOutputDemand::new("anchor-a"))
        .start_dependent_in_program::<NoReuseProgram, FinalConnection>(application)
        .unwrap();
    for _ in 0..64 {
        if let WorthQueryApplicationOutputDemandProgress::Settled(settled) =
            demand.advance(request).unwrap()
        {
            return settled;
        }
    }
    panic!("the real final demand must settle");
}

fn assert_values(request: &Request<'_, '_, '_>) {
    assert_root_value(request);
    let source = request
        .query(PlanarRead {
            body_key: "anchor-a".into(),
        })
        .execute()
        .unwrap();
    assert_eq!(source.rows().len(), 1);
    assert_eq!(source.rows()[0].y, support::length(9));
    let output = request
        .query(PlanarOutputRead {
            body_key: "final:anchor-a".into(),
        })
        .execute()
        .unwrap();
    assert_eq!(
        output.rows().len(),
        1,
        "Preserve must not create a second entity under the key"
    );
    assert_eq!(output.rows()[0].value, support::length(11));
}

fn assert_root_value(request: &Request<'_, '_, '_>) {
    let root = request
        .query(PlanarOutputRead {
            body_key: "anchor-a".into(),
        })
        .execute()
        .unwrap();
    assert_eq!(root.rows().len(), 1);
    assert_eq!(root.rows()[0].value, support::length(10));
}

fn verify_current<'a, 'p, 's>(
    request: &Request<'a, 'p, 's>,
    application: &'a Application,
    key: u64,
) -> Result<(), PlanarMutationDenial> {
    let source = request
        .query(PlanarRead {
            body_key: "anchor-a".into(),
        })
        .execute()
        .unwrap();
    match request.advanced().mutate(PlanarEdit(PlanarMutation {
        scope_key: "anchor-a".into(),
        operation: PlanarOperation::VerifyFinalCurrentOutputs(vec![PlanarCurrentOutputExpectation {
            producer_key: "anchor-a".into(), output_key: "final:anchor-a".into(),
        }]),
    })).expect_source(source.observed_sources()[0].clone()).idempotency(&key)
        .execute_in_program(application, ExecutionAllocationPolicy::SystemAllocation)
    {
        Ok(Outcome::Committed { .. }) => Ok(()),
        Ok(Outcome::DomainDenied(PlanarMutationDenial::CurrentOutputMissing)) => {
            Err(PlanarMutationDenial::CurrentOutputMissing)
        }
        other => panic!("the distinct verification command must commit or return exactly CurrentOutputMissing: {other:?}"),
    }
}

#[cfg(test)]
#[path = "build_identity/serial.rs"]
mod serial;
