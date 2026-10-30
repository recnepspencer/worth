use worth_store::physical_runtime::{RootPublicationCandidatePlan, RootPublicationPlanningMembers};

fn require_clone<T: Clone>() {}

fn main() {
    require_clone::<RootPublicationPlanningMembers>();
    require_clone::<RootPublicationCandidatePlan>();
}
