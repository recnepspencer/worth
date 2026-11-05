//! One request settles the wave; member outcomes travel to owner completion.
use {crate::domain_computation::primary_graph::vocabulary::application_contribution::prepared_request::ComputedSlot, crate::domain_computation::primary_graph::vocabulary::application_contribution::prepared_request::PreparedCompute};
use crate::domain_computation::primary_graph::vocabulary::application_contribution::member_execution::WorthQueryMemberExecution;
use worth_execution::{ExecutionMap, MapKernelFailure, MapOutcome, MapStop};
use worth_foundational::PartitionIdentity;
use {
    crate::domain_computation::request_execution::execution_stop::checkpoint_denial,
    crate::domain_computation::request_execution::execution_stop::lease_denial,
};
use {
    crate::domain_computation::request_execution::QueryRequestExecution,
    crate::domain_computation::request_execution::WorthQueryAdvancementDenial,
};
use crate::domain_computation::primary_graph::vocabulary::application_contribution::wave_dispatch::ComputedMember;
use crate::domain_computation::primary_graph::vocabulary::application_contribution::wave_dispatch::NoMemberFailure;
use crate::domain_computation::primary_graph::vocabulary::application_contribution::wave_dispatch::PreparedMember;
use crate::domain_computation::primary_graph::vocabulary::application_contribution::wave_dispatch::settle_outcome;


/// Interruption already present at dispatch has width-independent settlement.
/// An interruption arriving during compute can be seen by an earlier running
/// member: only the canonical-prefix safety law is promised across widths.
pub fn dispatch(
    execution: &QueryRequestExecution<'_>,
    members: Vec<PreparedMember>,
) -> Vec<Result<ComputedMember, WorthQueryAdvancementDenial>> {
    if members.iter().all(|member| member.requests.is_empty()) {
        return members
            .into_iter()
            .map(|_| Ok(ComputedMember { slots: Vec::new() }))
            .collect();
    }
    let wave = execution.wave();
    let partitions = members
        .into_iter()
        .enumerate()
        .map(|(rank, member)| (PartitionIdentity::new(rank as u64 + 1), member))
        .collect();
    let map = match ExecutionMap::<_, ()>::from_native_partitions(partitions) {
        Ok(map) => map,
        Err(kind) => {
            return vec![Err(WorthQueryAdvancementDenial::Resource(
                crate::domain_computation::vocabulary::request_execution::execution_refusal::WorthQueryManagedComputationResourceDenial::NativeAllocationRefused {
                    kind,
                    requested_payload_bytes: None,
                },
            ))]
        }
    };
    let request = match wave.request().execution_request() {
        Ok(request) => request,
        Err(denial) => return vec![Err(WorthQueryAdvancementDenial::Resource(denial))],
    };
    let outcome = request.in_scope(|lease| {
        let kernel = |input: PreparedMember,
                      context: &mut worth_execution::MapKernelContext<'_, '_>|
         -> Result<_, MapKernelFailure<NoMemberFailure>> {
            let member = wave.member();
            let mut slots = Vec::with_capacity(input.requests.len());
            for request in input.requests {
                // A prior request's untaken cause cannot reach a later
                // request, even when that request returns its own error.
                context.poll()?;
                let accounted = context.has_accounting();
                let context = std::cell::RefCell::new(&mut *context);
                let mut poll = || context.borrow_mut().poll();
                let mut charge = |units| context.borrow_mut().checkpoint(units);
                let mut take = || context.borrow_mut().take_nested_computation_stop();
                let slot = request.run(WorthQueryMemberExecution::for_kernel(
                    &member,
                    &mut poll,
                    accounted.then_some(&mut charge),
                    &mut take,
                ));
                slots.push(slot);
            }
            let result = ComputedMember { slots };
            Ok(result)
        };
        let outcome =
            match crate::domain_computation::request_execution::certified_wave_seed_for_test(
                wave.request(),
            ) {
                Some(seed) => {
                    let (outcome, order) =
                        map.run_owned_native_perturbed_for_test(lease, seed, kernel);
                    crate::domain_computation::primary_graph::vocabulary::application_contribution::seeded_dispatch_orders::record(order);
                    outcome
                }
                None => map.run_owned_native(lease, kernel),
            };
        settle_outcome(outcome)
    });
    match outcome {
        Ok(outcome) => outcome,
        Err(denial) => vec![Err(
            crate::domain_computation::request_execution::advancement_denial(denial),
        )],
    }
}



pub(crate) mod seeded_dispatch_orders;

mod court_doors_for_test;
