//! Crash-consistency oracle for one anchored-record publication.
//!
//! For the first value of an anchor and for a replacement, the oracle counts
//! the media operations of every role the publication performs, then reruns it
//! once per (role, ordinal, directive) with a fault at exactly that operation,
//! abandons the runtime, recovers in the production path and requires: the
//! anchor holds exactly the old or the new value (an acknowledged value is
//! never lost, a proven-no-effect one never appears); an indeterminate
//! publication's name refuses as unresolved until reopen; the journal beside it is
//! unchanged; the new value then publishes; and a second recovery agrees.

#[allow(unused_imports)]
use crate::production_entry::anchored_records_oracle::*;

#[test]
fn every_media_operation_of_an_anchored_publication_recovers_to_the_old_or_new_value() {
    let mut unresolved_refusals = 0;
    // The first value of the anchor (no anchor yet) and a replacement.
    for target in [1, 2] {
        let counts = measure(target);
        assert!(
            !counts.is_empty(),
            "the publication performed media operations"
        );
        for (role, count) in counts {
            for nth in 1..=count {
                let nth = std::num::NonZeroU64::new(nth).unwrap();
                for directive in [
                    worth_store::physical_runtime::certification::MediaFaultDirective::FailBefore {
                        kind: std::io::ErrorKind::Other,
                        raw_os_error: None,
                    },
                    worth_store::physical_runtime::certification::MediaFaultDirective::IndeterminateAfterEffect,
                ] {
                    unresolved_refusals += usize::from(run_case(target, role, nth, directive));
                }
            }
        }
    }
    assert!(
        unresolved_refusals > 0,
        "no indeterminate publication left a serving runtime to refuse its name"
    );
}
