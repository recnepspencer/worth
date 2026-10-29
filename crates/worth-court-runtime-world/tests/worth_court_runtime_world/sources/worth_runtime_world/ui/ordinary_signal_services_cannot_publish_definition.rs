use worth_signal::facade::branch::{
    AdmittedSignalBranchBasis, SignalConditionalDefinitionPublicationOperation,
    SignalOwnerCancellationToken, SignalOwnerServicePorts,
};

fn bypass_world(
    services: SignalOwnerServicePorts<(), (), (), (), ()>,
    publication: SignalConditionalDefinitionPublicationOperation,
    basis: AdmittedSignalBranchBasis,
    cancellation: &SignalOwnerCancellationToken,
) {
    let _ = services
        .mutation_port()
        .advance_conditional_definition_exact_with_completion(
            publication,
            &basis,
            &mut (),
            cancellation,
            |_| Ok(()),
        );
}

fn main() {}
