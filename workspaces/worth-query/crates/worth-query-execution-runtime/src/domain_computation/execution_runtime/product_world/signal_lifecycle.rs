//! An authored Home never receives a volatile Signal branch-retirement writer.
#[derive(Clone)]
pub enum SignalLifecycleServices {
    Memory(worth_signal::facade::branch::SignalBranchLifecyclePort<(), (), ()>),
    AuthoredHome,
}
