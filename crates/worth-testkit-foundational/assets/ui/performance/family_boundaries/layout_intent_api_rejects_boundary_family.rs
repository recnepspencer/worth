use worth_foundational::{
    foundational_performance_layout_intent_definitions, FoundationalPerformanceBoundary,
};

fn requires_layout(_: worth_foundational::FoundationalPerformanceLayoutIntent) {}

fn main() {
    let definitions = foundational_performance_layout_intent_definitions();
    let _ = definitions[0].family();
    requires_layout(FoundationalPerformanceBoundary::AuthoritativeExecution);
}
