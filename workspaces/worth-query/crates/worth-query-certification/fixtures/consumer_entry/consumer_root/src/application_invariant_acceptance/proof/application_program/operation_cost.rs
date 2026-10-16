//! Ordinary/native discovered work with independent required-set and retained-population axes.
use crate::application_invariant_acceptance::membership::{
    Fanout, FanoutRoot, MembershipProgram, MembershipSchema as Schema,
};
use crate::application_invariant_acceptance::{authentication, resources, seed::length};
use std::num::NonZeroUsize;
use worth_query_decl::facade::{
    application_operation::*, application_program::*, application_schema::*,
};
use worth_query_host::facade::{
    application_entry::*, application_installation::WorthQueryProgramApplicationRuntime,
};
use worth_query_topology_entry::*;
mod capture;
mod declaration;
mod installation;
mod journey;
mod measurement;
mod native;
mod population;
mod stops;
use declaration::{Action, Program};
type Runtime = WorthQueryProgramApplicationRuntime<Schema, Program>;
type Request<'a> = WorthQueryApplicationRequest<'a, 'a, 'a, Schema>;
type Operation = WorthQueryDiscoveredOperation<Schema, Original, Program, FanoutRoot, Action>;
type Settled = WorthQuerySettledDiscoveredOperation<Schema, Original, Program, FanoutRoot, Action>;

// Deliberately not Clone: retention and each pump must borrow the actual original.
struct Original(PlanarSourceAdjustment);
impl ApplicationMutationIntent<Schema> for Original {
    type Binding = PlanarSourceAdjustmentBinding<Schema>;
    fn input(&self) -> &PlanarSourceAdjustment {
        &self.0
    }
    fn scope_binding(&self) -> PlanarMutationScope<Schema> {
        <PlanarSourceAdjustment as ApplicationMutationIntent<Schema>>::scope_binding(&self.0)
    }
}
fn input(key: &str) -> PlanarSourceAdjustment {
    PlanarSourceAdjustment {
        scope_key: key.to_owned(),
        replacement_y: length(2),
    }
}
fn retain(application: &Runtime, request: &Request<'_>, body: &str, key: u64) -> Operation {
    let source = request
        .query(PlanarRead {
            body_key: body.to_owned(),
        })
        .execute()
        .unwrap()
        .observed_sources()[0]
        .clone();
    request
        .mutate(Original(input(body)))
        .expect_source(source)
        .preconditions(TypedMutationPreconditions::new().expect_fact(
            BodyKey::reference::<Schema>(),
            ApplicationEncodedScalarValue::try_new(body.to_owned()).unwrap(),
        ))
        .idempotency_owned(key)
        .unwrap_or_else(|(_, _, denial)| panic!("input capture: {denial:?}"))
        .retain_operation::<Program, Action>(application)
        .unwrap_or_else(|(_, denial)| panic!("exact cost descriptor: {denial:?}"))
}
fn settle(mut operation: Operation, request: &Request<'_>) -> Settled {
    // A test bound diagnoses a stuck pending owner; it never renews native policy.
    for _ in 0..256 {
        match operation.advance(request) {
            WorthQueryOperationAdvance::Progress {
                operation: next, ..
            } => operation = next,
            WorthQueryOperationAdvance::Settled(settled) => return settled,
            WorthQueryOperationAdvance::Refused { denial, .. }
            | WorthQueryOperationAdvance::Terminal { denial, .. } => {
                panic!("cost journey: {denial:?}")
            }
        }
    }
    panic!("native discovered continuation did not settle")
}
