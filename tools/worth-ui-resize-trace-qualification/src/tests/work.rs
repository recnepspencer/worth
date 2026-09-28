use crate::analysis::tests::host;
use crate::logs::HostKind;
use crate::work::{during, Work, STAGES};

#[test]
fn work_counts_only_the_drag() {
    let mut trace = host(&[
        (5, HostKind::Target([1536, 1024])),
        (3, HostKind::Swapchain([768, 768])),
        (10, HostKind::Target([900, 700])),
        (10, HostKind::Swapchain([1024, 768])),
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
        (
            25,
            HostKind::Stage {
                stage: 8,
                start: 21,
            },
        ),
        (26, HostKind::Stage { stage: 8, start: 5 }),
        (
            29,
            HostKind::Stage {
                stage: 0,
                start: 12,
            },
        ),
        (
            31,
            HostKind::Stage {
                stage: 0,
                start: 29,
            },
        ),
        (40, HostKind::Target([800, 600])),
    ]);
    let mut stage_ms: [Vec<f64>; STAGES.len()] = Default::default();
    stage_ms[0] = vec![17.0];
    stage_ms[8] = vec![4.0, 21.0];
    trace.peaks.push(("textures".to_owned(), 4));
    let work = during(&trace, [10, 30]);
    assert_eq!(
        work,
        Work {
            targets: vec![[900, 700]],
            swapchains: vec![[1024, 768]],
            text_attempts: 2,
            shaping_attempts: 1,
            text_total: [2, 40, 52, 4, 7],
            text_max: [2, 40, 40, 3, 7],
            stage_ms,
            peaks: vec![("textures".to_owned(), 4)],
        }
    );
}
