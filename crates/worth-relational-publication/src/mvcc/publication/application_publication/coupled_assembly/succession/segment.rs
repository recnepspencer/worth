//! Each member's contiguous entries in the candidate's shared accumulators
//! (#119). A superseded round's entries are removed when its chain retracts;
//! every member appended after them shifts down, so the accumulators always
//! hold exactly the live members' effects.
use std::ops::Range;
use worth_relational_mutation::internal_port::authority::mutation::MutationEffect;

/// One member's entries in `changed_records`, the parallel `patch_fragments`
/// and `canonical_deltas`, `adjacency.deltas` and `diagnostics.entries`.
#[derive(Clone, Debug, Default)]
pub struct MemberSpan {
    pub records: Range<usize>,
    pub fragments: Range<usize>,
    pub adjacency: Range<usize>,
    pub diagnostics: Range<usize>,
    /// Whether this member's own effect deleted an entity.
    pub has_entity_deletions: bool,
}

/// Accumulator lengths before a member appends.
#[derive(Clone, Copy, Debug, Default)]
pub struct SpanStart {
    records: usize,
    fragments: usize,
    adjacency: usize,
    diagnostics: usize,
}

impl SpanStart {
    pub fn of(effect: &MutationEffect) -> Self {
        Self {
            records: effect.publication.changed_records.len(),
            fragments: effect.publication.patch_fragments.len(),
            adjacency: effect.adjacency.deltas.len(),
            diagnostics: effect.diagnostics.entries.len(),
        }
    }

    /// The span a member appended since `self`.
    pub fn end(self, effect: &MutationEffect, has_entity_deletions: bool) -> MemberSpan {
        let end = Self::of(effect);
        MemberSpan {
            records: self.records..end.records,
            fragments: self.fragments..end.fragments,
            adjacency: self.adjacency..end.adjacency,
            diagnostics: self.diagnostics..end.diagnostics,
            has_entity_deletions,
        }
    }
}

impl MemberSpan {
    /// Remove this span's entries from `effect`. O(entries after the span).
    pub fn remove_from(&self, effect: &mut MutationEffect) {
        let publication = &mut effect.publication;
        publication.changed_records.drain(self.records.clone());
        publication.patch_fragments.drain(self.fragments.clone());
        publication.canonical_deltas.drain(self.fragments.clone());
        effect.adjacency.deltas.drain(self.adjacency.clone());
        effect.diagnostics.entries.drain(self.diagnostics.clone());
    }

    /// Shift this span down by `removed`'s entries on every axis where it lies
    /// after `removed`. Spans before it are unchanged.
    pub fn shift_after(&mut self, removed: &Self) {
        fn shift(range: &mut Range<usize>, removed: &Range<usize>) {
            if range.start >= removed.end && !removed.is_empty() {
                let width = removed.len();
                *range = range.start - width..range.end - width;
            }
        }
        shift(&mut self.records, &removed.records);
        shift(&mut self.fragments, &removed.fragments);
        shift(&mut self.adjacency, &removed.adjacency);
        shift(&mut self.diagnostics, &removed.diagnostics);
    }
}

/// Every live member's span: the authored prefix, each stage, and each
/// chain's live round except part `except`'s.
pub fn live_spans(
    prefix: &MemberSpan,
    stages: &[super::super::retained_assembly::CoupledAuthoredStagePart],
    parts: &[super::super::retained_assembly::CoupledPublicationPart],
    except: Option<usize>,
) -> Vec<MemberSpan> {
    let parts = parts
        .iter()
        .enumerate()
        .filter(|(index, _)| Some(*index) != except);
    std::iter::once(prefix.clone())
        .chain(stages.iter().map(|stage| stage.span.clone()))
        .chain(parts.map(|(_, part)| part.chain.span.clone()))
        .collect()
}
