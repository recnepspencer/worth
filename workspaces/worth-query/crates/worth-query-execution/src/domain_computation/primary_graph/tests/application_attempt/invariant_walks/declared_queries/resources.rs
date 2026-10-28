//! Source custody survives the result and releases with its last owner.
use super::*;
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};
#[test]
fn complete_declared_path_retains_original_memory_until_last_source() {
    let world = diamond(6, vec![]);
    let payer = SerialMemoryBudget::new(None);
    let host = SerialRequest::from_memory(payer.clone(), CancellationSource::new().token(), None);
    let request = crate::domain_computation::primary_graph::tests::fixture::live_scope();
    let result = read::execute(
        &world,
        &ExecutionRequest::serial(&host),
        &request,
        Forward::reference(),
        0,
    )
    .unwrap();
    assert!(payer.charged_memory_bytes() > 0);
    let full = payer.charged_memory_bytes();
    let body = std::sync::Arc::clone(
        &result.observed_sources()[0]
            .footprint_for_test()
            .root_selection
            .as_ref()
            .unwrap()
            .walks
            .as_ref()
            .unwrap()
            .values()[0],
    );
    let retained = result.observed_sources()[0].clone();
    drop(result);
    assert!(
        payer.charged_memory_bytes() > 0,
        "one real source still retains body and source backing"
    );
    let source_held = payer.charged_memory_bytes();
    drop(retained);
    assert!(
        payer.charged_memory_bytes() > 0,
        "the separately retained real Native body remains funded"
    );
    assert!(
        payer.charged_memory_bytes() < source_held,
        "source metadata payment is distinct and releases before Native body payment"
    );
    drop(body);
    assert_eq!(
        payer.charged_memory_bytes(),
        0,
        "last owner returns original payment"
    );
    // Calibrate against actual original custody, requiring the new allocation
    // boundary rather than assuming a final-resident limit hits that boundary.
    let mut source_backing_refused = false;
    let mut observed_boundaries = Vec::new();
    for limit in [full - 1, source_held - 1, source_held / 2] {
        let finite = SerialMemoryBudget::new(Some(limit));
        let limited =
            SerialRequest::from_memory(finite.clone(), CancellationSource::new().token(), None);
        match read::execute(
            &world,
            &ExecutionRequest::serial(&limited),
            &request,
            Forward::reference(),
            0,
        ) {
            Ok(result) => {
                drop(result);
                observed_boundaries.push((limit, "complete".to_owned()));
            }
            Err(refused) => {
                observed_boundaries.push((
                    limit,
                    format!("{:?}: {}", refused.kind(), refused.subject()),
                ));
                source_backing_refused |= refused.subject() == "closure source backing";
            }
        }
        assert_eq!(
            finite.charged_memory_bytes(),
            0,
            "every actual refusal/success releases original custody"
        );
        if source_backing_refused {
            break;
        }
    }
    eprintln!("finite source backing boundaries: {observed_boundaries:?}");
    assert!(
        source_backing_refused,
        "an actual finite original payer must refuse the new source allocation"
    );
    let ample = SerialMemoryBudget::new(Some(full * 2));
    let funded = SerialRequest::from_memory(ample.clone(), CancellationSource::new().token(), None);
    let twin = read::execute(
        &world,
        &ExecutionRequest::serial(&funded),
        &request,
        Forward::reference(),
        0,
    )
    .unwrap();
    assert_eq!(keys(&twin), [0, 1, 2, 3, 4]);
    drop(twin);
    assert_eq!(ample.charged_memory_bytes(), 0);
}
