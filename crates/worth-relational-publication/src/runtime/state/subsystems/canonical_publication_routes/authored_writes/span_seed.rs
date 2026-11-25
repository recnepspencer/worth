//! Span-bounded ordinary seed of the discardable authored-write generation.
//!
//! A reopened runtime holds no generation: checkpoint artifacts and replayed
//! commits get none, and every later commit inherits that absence. A history
//! cursor needs only the original and the commits after it, because every
//! admission check reads ordinals strictly after the original. So this seed
//! walks parents from the target back to the original and extends a fresh
//! generation over exactly that span, with the same per-commit extension the
//! commit path performs.
//!
//! - **Bound.** O(span × log span + authored writes in span × log keys), where
//!   span counts the commits from the original through the target. It never
//!   depends on the history before the original. The walk stops at the
//!   original, at the root (`NotSelectedPrefix`), at a merge
//!   (`UnsupportedMergeInterval`), or after the caller's link bound
//!   (`SpanBoundReached`). Every step is charged to the original request.
//! - **Widen-only.** The install happens under the residency lock, and only
//!   when the target's generation lacks the original, so a narrower seed never
//!   replaces a wider one.
//! - **Correct or absent.** The seed is built from canonical envelopes only and
//!   becomes visible once complete. It is never persisted, so a crash leaves it
//!   absent.
//!
//! This is not cold reconstruction. `rebuild_authored_generation` remains the
//! explicit whole-ancestry lane, and this seed never walks past the original.
use branch::history::access::relevant_writes::generation::AuthoredWriteGeneration;
use branch::history::access::relevant_writes::{
    RelationalAuthoredWriteDenial as Denial, RelationalAuthoredWriteSpan as Span,
};
use std::num::NonZeroUsize;
use worth_execution::{ExecutionOrderedMap, ExecutionRequest};
use worth_relational_branch::internal_port as branch;
use worth_relational_contracts::internal_port::history::data::coordinates::CommitId;

impl super::super::RelationalCanonicalPublicationRoutes {
    pub(crate) fn seed_authored_span(
        &self,
        execution: ExecutionRequest<'_, '_>,
        original: CommitId,
        selected: CommitId,
        maximum_links: Option<NonZeroUsize>,
    ) -> Result<Span, Denial> {
        let owner = self
            .authored_generation_owner(selected)
            .ok_or(Denial::CanonicalEvidenceUnavailable)?;
        if let Some(current) = owner.authored_write_generation() {
            if current.unsupported_merge {
                return Err(Denial::UnsupportedMergeInterval);
            }
            if current.prefix.get(&original).is_some() {
                return Ok(Span::Covered);
            }
        }
        let (generation, commits) =
            branch::history::access::relevant_writes::run(execution, |context| {
                let mut chain = ExecutionOrderedMap::<u64, CommitId>::default();
                let mut next = selected;
                let mut links = 0usize;
                loop {
                    branch::history::access::relevant_writes::visit(context)?;
                    branch::history::access::relevant_writes::ordered_visit(context, chain.len())?;
                    chain.insert_cloned_native_for_request(
                        &(chain.len() as u64),
                        &next,
                        execution,
                    )?;
                    if next == original {
                        break;
                    }
                    if maximum_links.is_some_and(|maximum| links >= maximum.get()) {
                        return Err(Denial::SpanBoundReached);
                    }
                    let envelope = self
                        .demand_envelope(&execution, next)
                        .map_err(|cause| Denial::Execution(cause.into()))?
                        .ok_or(Denial::CanonicalEvidenceUnavailable)?;
                    next = match envelope.commit.parents.as_slice() {
                        [] => return Err(Denial::NotSelectedPrefix),
                        [parent] => *parent,
                        _ => return Err(Denial::UnsupportedMergeInterval),
                    };
                    links = links.checked_add(1).ok_or(Denial::SizeOverflow)?;
                }
                let mut generation = AuthoredWriteGeneration::default();
                for rank in (0..chain.len()).rev() {
                    branch::history::access::relevant_writes::visit(context)?;
                    branch::history::access::relevant_writes::ordered_visit(context, chain.len())?;
                    let (_, commit) = chain.entry_at(rank).expect("retained span rank");
                    let envelope = self
                        .demand_envelope(&execution, *commit)
                        .map_err(|cause| Denial::Execution(cause.into()))?
                        .ok_or(Denial::CanonicalEvidenceUnavailable)?;
                    super::extend_generation(&mut generation, execution, context, &envelope)?;
                }
                Ok((generation, chain.len()))
            })?;
        if !owner.install_authored_write_span(original, generation) {
            return Ok(Span::Covered);
        }
        Ok(Span::Seeded { commits })
    }
}
