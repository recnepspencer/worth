//! Multi-read completion retains the original V1 pin through a real V2 adoption.
use super::*;
use crate::document_retention_model::operator_identity::request_scope;
use std::{cell::Cell, num::NonZeroUsize};
use worth_execution::{CancellationSource, ExecutionRequest, ExecutionWorkCeiling, LeaseRequest};
use worth_foundational::{ExecutionBudget, ExecutionRequestPolicy};
use worth_query_host::facade::{
    declaration::application_schema::ApplicationScalarValueBinding, primary_graph::*,
    runtime::ExecutionAllocationPolicy,
};
#[test]
fn changed_codec_retained_snapshot_keeps_original_pin_and_latched_completion() {
    let (original, authority) = host::install_with_projection(journey::heavy()).unwrap();
    let _settled = host::produce(&original);
    let runtime = original.runtime();
    let scope = request_scope();
    let projected = runtime
        .with_application_advancement(&scope, |phase| {
            authority.project(
                &phase,
                |reader| {
                    ["derived-report", "derived-left"].map(|name| {
                        reader
                            .resolve_entity(
                                schema::DerivedIdentityField::reference(),
                                name.to_owned(),
                            )
                            .unwrap()
                    })
                },
                ExecutionAllocationPolicy::SystemAllocation,
            )
        })
        .unwrap()
        .unwrap();
    let (identities, snapshot, _) = projected.into_parts();
    let version = snapshot.version();
    let target = journey::successor(host::capture(&original));
    assert_eq!(successor::output::decodes(), 0);
    host::edit_source(&original, journey::edited(), 180501);
    assert_eq!(output::literals(&host::read_derived(&original)).1, 37);
    runtime
        .with_application_advancement(&scope, |phase| {
            let execution = phase
                .checked_execution_request_for(runtime.product_runtime())
                .unwrap();
            let run = |work, cancel| {
                output::reset_decodes();
                let first_read = Cell::new(false);
                let returned = Cell::new(false);
                let cancellation = CancellationSource::new();
                let child = execution
                    .in_scope(|parent| {
                        let parent = parent.expect("original admitted snapshot lease");
                        parent
                            .child(LeaseRequest {
                                policy: ExecutionRequestPolicy::new(
                                    parent.policy().posture(),
                                    parent.policy().determinism(),
                                    ExecutionBudget::new(
                                        NonZeroUsize::MIN,
                                        Some(64 * 1024 * 1024),
                                        Some(work),
                                    ),
                                ),
                                cancellation: cancellation.token(),
                                deadline: scope.deadline(),
                            })
                            .unwrap()
                    })
                    .unwrap();
                let request = ExecutionRequest::leased(&child);
                let (result, report) = request
                    .run(ExecutionWorkCeiling::new(Some(work)), |_| {
                        snapshot.read_fields(&request, |reader| {
                            let first = reader
                                .field(&identities[0], schema::DerivedValueField::reference())
                                .unwrap()
                                .unwrap();
                            assert_eq!(output::literals(&first).1, 96);
                            first_read.set(true);
                            let second = reader
                                .field(&identities[1], schema::DerivedValueField::reference())
                                .ok()
                                .flatten();
                            if cancel {
                                cancellation.cancel();
                            }
                            returned.set(true);
                            if cancel {
                                Err("retained-domain-rejection")
                            } else {
                                Ok((first, second))
                            }
                        })
                    })
                    .unwrap();
                (result, first_read.get(), returned.get(), report)
            };
            let (funded, first, returned, report) = run(8_000_000, false);
            let (a, b) = funded.unwrap().unwrap();
            assert!(first && returned);
            assert_eq!(output::literals(&a).1, 96);
            assert_eq!(output::literals(&b.unwrap()).1, 96);
            assert_eq!(output::ignored_refusals(), 0);
            let worth_foundational::facade::AspectValue::String(
                worth_foundational::facade::InternedString::Raw(wire),
            ) = output::DerivedBinding::encode(&output::calculate(&journey::heavy())).unwrap()
            else {
                panic!("canonical carrier")
            };
            let narrow = report
                .charged_work()
                .checked_sub(u64::try_from(wire.len() * 3 / 2).unwrap())
                .unwrap();
            let (denied, first, returned, _) = run(narrow, true);
            assert!(first && returned);
            assert!(output::ignored_refusals() > 0);
            assert_eq!(
                denied.err().unwrap().kind(),
                WorthQueryInvariantProjectionDenialKind::ValueDecode(
                    WorthQueryValueDecodeDenial::Checkpoint(
                        worth_execution::MapKernelStop::WorkCeiling
                    )
                )
            );
            let (cancelled, first, returned, _) = run(8_000_000, true);
            assert!(first && returned);
            assert_eq!(
                cancelled.err().unwrap().kind(),
                // The final handoff reports the original request's own stop.
                WorthQueryInvariantProjectionDenialKind::ExecutionRequest(
                    worth_relational::facade::transactions::RelationalExecutionDenialCause::Cancelled
                )
            );
            let (retry, first, returned, _) = run(8_000_000, false);
            assert!(retry.unwrap().is_ok());
            assert!(first && returned);
            assert_eq!(snapshot.version(), version);
        })
        .unwrap();
    assert_eq!(successor::output::decodes(), 0);
    assert!(runtime.close_live_delivery().owner_terminal());
    let _new = successor::host::produce(&target);
    assert_eq!(
        successor::output::literals(&successor::host::read_derived(&target)).1,
        1096
    );
    runtime
        .with_application_advancement(&scope, |phase| {
            let execution = phase
                .checked_execution_request_for(runtime.product_runtime())
                .unwrap();
            let value = snapshot
                .field(
                    &execution,
                    &identities[0],
                    schema::DerivedValueField::reference(),
                )
                .unwrap()
                .unwrap();
            assert_eq!(output::literals(&value).1, 96);
        })
        .unwrap();
}
