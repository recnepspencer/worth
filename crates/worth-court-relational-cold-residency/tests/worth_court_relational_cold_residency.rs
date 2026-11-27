#[global_allocator]
static ALLOCATOR: tracking_allocator::Allocator<std::alloc::System> =
    tracking_allocator::Allocator::system();

#[path = "worth_court_relational_cold_residency/relational/mod.rs"]
mod relational;
