pub use super::*;
pub use worth_query_host::facade::{
    application_entry::{
        WorthQueryApplicationProgramAdoptionPreparationDenial as Preparation,
        WorthQueryApplicationProgramInspectionDenial as Inspection,
    },
    primary_graph::{
        installed_source_reads_on_this_thread_for_test as reads,
        WorthQueryBranchAdoptionPreparationDenial as Adoption,
    },
};

pub fn preparation_cause(denial: Preparation) -> Denial {
    match denial {
        Preparation::Adoption(Adoption::ExecutionDenied(cause)) => cause,
        other => panic!("preparation must refuse at open: {other:?}"),
    }
}
