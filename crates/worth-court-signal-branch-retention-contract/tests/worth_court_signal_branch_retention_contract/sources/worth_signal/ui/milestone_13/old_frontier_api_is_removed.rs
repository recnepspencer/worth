use worth_signal::facade::adapters as surface;

fn main() {
    let _ = surface::FrontierPredictedCounters::default();
    let _ = std::mem::size_of::<surface::FrontierPlan>();
    let _ = std::mem::size_of::<surface::TransitiveFrontierWaveSummary>();
}
