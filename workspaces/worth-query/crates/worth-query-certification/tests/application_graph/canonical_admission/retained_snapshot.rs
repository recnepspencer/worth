//! Multi-read completion keeps one decode grant and the original retained pin.
use super::{host, opaque, operation::Reads, schema::*};
use crate::document_retention_model::operator_identity::request_scope;
use std::{cell::Cell, num::NonZeroUsize};
use worth_execution::{CancellationSource, ExecutionRequest, ExecutionWorkCeiling, LeaseRequest};
use worth_foundational::{ExecutionBudget, ExecutionRequestPolicy};
use worth_query_host::facade::{
    declaration::application_schema::ApplicationScalarValueBinding, primary_graph::*,
    runtime::ExecutionAllocationPolicy,
};

#[test]
fn retained_snapshot_multi_read_vetoes_swallowed_refusal_and_retries_at_original_pin() {
    let (host, authority) = host::install_with_projection(super::FUNDED_WORK);
    let runtime = host.runtime();
    let scope = request_scope();
    let completed = runtime
        .with_application_advancement(&scope, |phase| {
            authority.project(
                &phase,
                |reader| {
                    ["root", "child"].map(|name| {
                        reader
                            .resolve_entity(DocumentIdentityField::reference(), name.to_owned())
                            .unwrap()
                    })
                },
                ExecutionAllocationPolicy::SystemAllocation,
            )
        })
        .unwrap()
        .unwrap();
    let (identities, snapshot, _) = completed.into_parts();
    let pinned_version = snapshot.version();

    // Advance the real native source while the observation remains pinned.
    // This is an ordinary installed command, not a relabelled snapshot or an
    // edited checkpoint. The later reads must still see the original counter.
    super::assert_committed(host::advance(&host, Reads::Root, 71_401), 1);
    assert_eq!(host::read(&host, "root").counter, 1);

    runtime
        .with_application_advancement(&scope, |phase| {
            let execution = phase
                .checked_execution_request_for(runtime.product_runtime())
                .unwrap();
            let run = |work, cancel_at_handoff| {
                opaque::reset();
                let first_read = Cell::new(false);
                let callback_returned = Cell::new(false);
                let cancellation = CancellationSource::new();
                let lease = execution
                    .in_scope(|parent| {
                        let parent = parent.expect("the native host lends its original lease");
                        parent
                            .child(LeaseRequest {
                                policy: ExecutionRequestPolicy::new(
                                    parent.policy().posture(),
                                    parent.policy().determinism(),
                                    ExecutionBudget::new(NonZeroUsize::MIN, 64 * 1024 * 1024, work),
                                ),
                                cancellation: cancellation.token(),
                                deadline: scope.deadline(),
                            })
                            .unwrap()
                    })
                    .unwrap();
                let request = ExecutionRequest::leased(&lease);
                let (result, report) = request
                    .run(ExecutionWorkCeiling::new(work), |_| {
                        snapshot.read_fields(&request, |reader| {
                            let first = reader
                                .field(&identities[0], DocumentValueField::reference())
                                .expect("the first nested decode fits this native allowance")
                                .expect("the pinned source has its root value");
                            assert_eq!(first.0, super::value());
                            first_read.set(true);
                            // The opaque seed can swallow its callback refusal;
                            // the caller can also swallow the field error. The
                            // native completion owner must veto either case.
                            let second = reader
                                .field(&identities[1], DocumentValueField::reference())
                                .ok()
                                .flatten();
                            if cancel_at_handoff {
                                cancellation.cancel();
                            }
                            callback_returned.set(true);
                            (first, second)
                        })
                    })
                    .expect("the real original execution scope ran the pinned read");
                (result, first_read.get(), callback_returned.get(), report)
            };

            let (funded, first, returned, report) = run(super::FUNDED_WORK, false);
            let (root, child) = funded.expect("funded cumulative retained reads complete");
            assert!(first && returned);
            assert_eq!(root.0, super::value());
            assert_eq!(child.unwrap().0, super::value());
            assert_eq!(opaque::ignored_refusals(), 0);

            let worth_foundational::facade::AspectValue::String(
                worth_foundational::facade::InternedString::Raw(wire),
            ) = super::binding::NestedBinding::encode(&opaque::SwallowingValue(super::value())).unwrap()
            else { panic!("the installed canonical value uses a string carrier"); };
            // The real native bill ends with the decoder's wire.len() canonical
            // comparison. Removing that plus half a wire reaches the last seed:
            // its sixteen 256-byte tags alone exceed this remaining tail.
            // Independent assertions below require the first read and a genuine
            // swallowed seed refusal, rather than accepting a lookup stop.
            let narrow_work = report.charged_work()
                .checked_sub(u64::try_from(wire.len() * 3 / 2).unwrap()).unwrap();
            // The later cancellation cannot replace the seed's first refusal.
            let (refused, first, returned, _) = run(narrow_work, true);
            let refused = refused.expect_err("swallowed second decode cannot publish output");
            assert!(first && returned);
            assert!(
                opaque::ignored_refusals() > 0,
                "the actual nested seed swallowed the real native refusal; funded={}, narrow={narrow_work}, wire={}", report.charged_work(), wire.len()
            );
            assert_eq!(
                refused.kind(),
                WorthQueryInvariantProjectionDenialKind::ValueDecode(
                    WorthQueryValueDecodeDenial::Checkpoint(
                        worth_execution::MapKernelStop::WorkCeiling
                    )
                )
            );

            let (cancelled, first, returned, _) = run(super::FUNDED_WORK, true);
            assert!(first && returned);
            assert_eq!(
                cancelled
                    .expect_err("late cancellation vetoes the completed callback")
                    .kind(),
                WorthQueryInvariantProjectionDenialKind::ValueDecode(
                    WorthQueryValueDecodeDenial::Scope(
                        worth_execution::WorkCeilingDenial::Stopped(
                            worth_execution::MapKernelStop::Cancelled
                        )
                    )
                )
            );

            let (retried, first, returned, _) = run(super::FUNDED_WORK, false);
            let (root, child) = retried.expect("a fresh request reuses the same native pin");
            assert!(first && returned);
            assert_eq!(root.0, super::value());
            assert_eq!(child.unwrap().0, super::value());
            assert_eq!(snapshot.version(), pinned_version);
            assert_eq!(
                snapshot
                    .field(&execution, &identities[0], CounterField::reference())
                    .unwrap(),
                Some(0),
                "the native retained pin keeps its old value after a committed source edit"
            );
        })
        .unwrap();
}
