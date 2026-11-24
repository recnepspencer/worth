//! A correction and coupled stages in one envelope outside the document: the
//! corrective stage under a real correction identity first, then the caller's
//! stages, all in one commit (plan §8a.7, S2's first court).
use super::envelope::{settle, Receipt, Undo};
use super::program::{self, Shape};
use super::reads::Host;
use super::*;
use std::num::NonZeroUsize;
use worth_query_host::facade::product::WorthQueryProductBranch;
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;

/// What one correction envelope with coupled stages did.
#[derive(Debug)]
pub enum Coupled {
    Committed,
    /// A stage's handler refused; nothing committed.
    DomainDenied,
    /// Any other outcome or execution denial, rendered for the court.
    Refused(String),
}

/// Reverse `undo` of `source` with `coupled` pushed after the corrective stage
/// in the same envelope on `branch`, then settle the program's demand there.
pub fn correct_with_coupled(
    host: &Host,
    shape: Shape,
    branch: WorthQueryProductBranch,
    key: String,
    source: &Receipt,
    undo: Undo,
    coupled: &[Call],
) -> Coupled {
    super::reads::with_branch_request(host, branch, |request| {
        let limits = WorthQueryCorrectionHistoryLimits::new(None, NonZeroUsize::MIN);
        let selected = request
            .corrections()
            .select(source, limits)
            .unwrap_or_else(|denial| panic!("{key}: the source must select: {denial:?}"));
        let reverse = request.corrections().reverse(&selected).idempotency(key.as_str()).limits(limits);
        macro_rules! run {
            ($correction:ty) => {{
                let mut envelope = reverse
                    .edit_envelope::<$correction, program::Program, ApplicationNoOutputGraph>(host)
                    .unwrap_or_else(|denial| panic!("{key}: the correction must prepare: {denial:?}"))
                    .ordered_authored()
                    .unwrap_or_else(|denial| panic!("{key}: the correction must order: {denial:?}"));
                for call in coupled {
                    dispatch!(*call, intent => envelope.push_stage::<ApplicationNoOutputGraph, _>(intent))
                        .unwrap_or_else(|denial| panic!("{key}: a coupled stage must push: {call:?}: {denial:?}"));
                }
                envelope.execute_performed(ExecutionAllocationPolicy::SystemAllocation)
            }};
        }
        let executed = match undo {
            Undo::Create(E1) => run!(UndoCreateE1),
            Undo::Create(_) => run!(UndoCreateE2),
            Undo::Retire(E1) => run!(UndoRetireE1),
            Undo::Retire(_) => run!(UndoRetireE2),
        };
        match executed {
            Ok(WorthQueryEditEnvelopeOutcome::Committed(_)) => {
                settle(host, request, shape);
                Coupled::Committed
            }
            Ok(WorthQueryEditEnvelopeOutcome::DomainDenied(_)) => Coupled::DomainDenied,
            Ok(other) => Coupled::Refused(format!("{other:?}")),
            Err(denial) => Coupled::Refused(format!("{denial:?}")),
        }
    })
}
