//! Ordinary program installation retains the actual topology producer/readiness owners.
use super::*;
use std::sync::{
    atomic::{AtomicBool, AtomicUsize},
    Arc,
};

pub(super) fn install<const MODE: u8>() -> Result<
    application_installation::WorthQueryProgramApplicationRuntime<JointSchema, JointProgram<MODE>>,
    WorthQueryInMemoryApplicationDenial,
> {
    let program = ApplicationProgramAuthoring::<JointSchema, JointProgram<MODE>>::begin()
        .validated_program()
        .expect("each guard control is a well-formed program");
    let declaration = JointSchema::declaration().expect("the real topology schema validates");
    application_installation::in_memory_program(
        program,
        declaration,
        (TopologyConfiguration {
            setup_calls: Arc::new(AtomicUsize::new(0)),
            invariant_calls: Arc::new(AtomicUsize::new(0)),
            invariant_probe: Arc::new(AtomicUsize::new(0)),
            producer_authorization_denials: Arc::new(AtomicUsize::new(0)),
            producer_domain_denial: Arc::new(AtomicBool::new(false)),
        },),
        support::limits(
            32,
            support::invalidation(128 * 1_024 * 1_024, 1_000_000, 32),
        ),
        |_phase, graph, installed| support::bind_model_owner(graph, installed),
    )
}
