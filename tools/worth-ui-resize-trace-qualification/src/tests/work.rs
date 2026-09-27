use crate::analysis::tests::host;
use crate::logs::HostKind;
use crate::work::{during, Work};

#[test]
fn work_counts_only_the_drag() {
    let mut trace = host(&[
        (5, HostKind::Target([1536, 1024])),
        (10, HostKind::Target([900, 700])),
        (
            11,
            HostKind::Text {
                frame: 1,
                work: [2, 40, 40, 3, 7],
            },
        ),
        (
            20,
            HostKind::Text {
                frame: 2,
                work: [0, 0, 12, 1, 0],
            },
        ),
        (40, HostKind::Target([800, 600])),
    ]);
    trace.peaks.push(("textures".to_owned(), 4));
    let work = during(&trace, [10, 30]);
    assert_eq!(
        work,
        Work {
            targets: vec![[900, 700]],
            text_attempts: 2,
            shaping_attempts: 1,
            text_total: [2, 40, 52, 4, 7],
            text_max: [2, 40, 40, 3, 7],
            peaks: vec![("textures".to_owned(), 4)],
        }
    );
}
