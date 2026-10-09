use super::*;
use worth_query_host::facade::{
    application_entry::{
        WorthQueryApplicationProgramAdoptionPreparationDenial as Preparation,
        WorthQueryApplicationProgramInspectionDenial as Inspection,
    },
    primary_graph::{
        installed_source_reads_on_this_thread_for_test as reads,
        WorthQueryBranchAdoptionPreparationDenial as Adoption,
    },
};

#[test]
fn program_entries_refuse_before_the_reader_used_by_their_admitted_control() {
    let host = publish_on_first_program();
    let branch = host.current_world();
    let target = *host
        .supported_program::<RetentionProgramP1>()
        .unwrap()
        .owned_revision();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let programs = || {
        host.runtime()
            .request(&principal, &scope)
            .on_branch(branch)
            .programs()
    };
    let requirements = programs().compare(&target).unwrap();
    for placement in [Placement::Serial, Placement::Leased(NonZeroUsize::MIN)] {
        let _restore = Restore(place(placement), bound(None));
        for zero_memory in [false, true] {
            for entry in 0..4 {
                bound(None);
                let before = reads();
                match entry {
                    0 => {
                        programs().compare(&target).unwrap();
                    }
                    1 => {
                        programs().inspect().unwrap();
                    }
                    2 => {
                        programs()
                            .adopt(&requirements)
                            .workflow_inventory(64)
                            .unwrap();
                    }
                    3 => {
                        drop(programs().adopt(&requirements).prepare(64).unwrap());
                    }
                    _ => unreachable!(),
                }
                assert!(
                    reads() > before,
                    "admitted entry {entry} must contact its reader"
                );
                bound(Some(worth_foundational::ExecutionBudget::new(
                    NonZeroUsize::MIN,
                    if zero_memory { 0 } else { 64 * 1024 * 1024 },
                    if zero_memory { 8_000_000 } else { 0 },
                )));
                let before = reads();
                let cause = match entry {
                    0 => preparation_cause(programs().compare(&target).err().unwrap()),
                    1 => match programs().inspect().err().unwrap() {
                        Inspection::ExecutionRequest(cause) => cause,
                        other => panic!("inspection must refuse at open: {other:?}"),
                    },
                    2 => preparation_cause(
                        programs()
                            .adopt(&requirements)
                            .workflow_inventory(64)
                            .err()
                            .unwrap(),
                    ),
                    3 => preparation_cause(
                        programs().adopt(&requirements).prepare(64).err().unwrap(),
                    ),
                    _ => unreachable!(),
                };
                assert_eq!(reads(), before, "refused entry {entry} reached its reader");
                if !zero_memory {
                    assert_eq!(cause, Denial::Resource(Resource::WorkExhausted));
                } else if placement == Placement::Serial {
                    assert_eq!(cause, Denial::Resource(Resource::PolicyMemoryLimit));
                } else {
                    let Denial::Resource(Resource::MemoryLimit {
                        level,
                        requested,
                        admitted,
                    }) = cause
                    else {
                        panic!("leased refusal must retain bytes: {cause:?}");
                    };
                    assert_eq!(level, Level::Policy);
                    assert!(requested > 0);
                    assert_eq!(admitted, 0);
                }
            }
        }
    }
}

fn preparation_cause(denial: Preparation) -> Denial {
    match denial {
        Preparation::Adoption(Adoption::ExecutionDenied(cause)) => cause,
        other => panic!("preparation must refuse at open: {other:?}"),
    }
}
