use worth_signal::facade::ResourceDiagnosticsExpansionBudget;

fn main() {
    let _forged = ResourceDiagnosticsExpansionBudget {
        allow_cold_reconstruction: true,
        max_replay_reconstruction_width: 1,
    };
}
