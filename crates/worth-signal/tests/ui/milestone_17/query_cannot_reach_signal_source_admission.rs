use worth_signal::facade::branch::SignalConditionalExecutionPort;

fn query_lane(
    port: &SignalConditionalExecutionPort<(), (), ()>,
    source: worth_proof::AdmittedConditionalSourceObservation,
) {
    let _ = port.admit_source(source);
}

fn main() {}
