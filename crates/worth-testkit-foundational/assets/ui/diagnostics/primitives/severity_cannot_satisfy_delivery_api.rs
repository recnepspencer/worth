use worth_foundational::{
    FoundationalDiagnosticDeliveryClass, FoundationalDiagnosticSeverity,
};

fn needs_delivery(_delivery: FoundationalDiagnosticDeliveryClass) {}

fn main() {
    let severity = FoundationalDiagnosticSeverity::Warning;
    needs_delivery(severity);
}
