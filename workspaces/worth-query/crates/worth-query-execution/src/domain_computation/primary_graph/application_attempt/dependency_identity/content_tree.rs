//! Deterministic content over logical ordinals, paired with its exact body.
use super::super::retained_decision_facts::{StorageControl, StoreDenial};
use super::{content_rule as rule, output_postcondition::is_output_currentness_fact};
use crate::domain_computation::primary_graph::application_checkpoint::facts::FactPayload;
use std::sync::Arc;
use worth_execution::{
    CanonicalBits, ChargedBytes, ExecutionMemoryReservation, ReductionPlan, ReductionTree,
};
use worth_foundational::facade::PartitionIdentity;
use worth_query_installation::facade::WorthQueryCanonicalWorkEvidence as Work;

mod admission;
mod fields;
mod postcondition;
use admission::{admit_tree, leaf};
#[cfg(test)]
mod oracle;

const VALUE_BOUND: u64 = 128;
type Tree = ReductionTree<Summary, fn(&Summary, &Summary) -> Summary>;

#[derive(Clone, Debug)]
pub(in crate::domain_computation::primary_graph) struct FactContent(Arc<Content>);

struct Content {
    body: FactPayload,
    tree: Tree,
    pair_work: Work,
    fields: Arc<fields::Fields>,
    // No old-root chain: this root funds its complete shared-tree view.
    _memory: Option<ExecutionMemoryReservation>,
}
impl std::fmt::Debug for Content {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Content")
            .field("facts", &self.body.len())
            .finish_non_exhaustive()
    }
}
impl PartialEq for FactContent {
    fn eq(&self, other: &Self) -> bool {
        self.0.body == other.0.body && self.0.tree.result() == other.0.tree.result()
    }
}
impl Eq for FactContent {}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Summary {
    digest: Result<[u8; 32], ()>,
    content_bytes: Option<usize>,
    reusable: bool,
}
impl ChargedBytes for Summary {
    fn additional_charged_bytes(&self) -> u64 {
        0
    }
}
impl CanonicalBits for Summary {
    fn canonical_len(&self) -> Option<usize> {
        Some(51)
    }
    fn visit_canonical_bits(&self, visit: &mut dyn FnMut(&[u8]) -> bool) -> bool {
        visit(&[
            u8::from(self.digest.is_ok()),
            u8::from(self.content_bytes.is_some()),
            u8::from(self.reusable),
        ]) && visit(&self.digest.unwrap_or([0; 32]))
            && visit(&(self.content_bytes.unwrap_or(0) as u128).to_le_bytes())
    }
}

fn combine(left: &Summary, right: &Summary) -> Summary {
    let derived = left
        .digest
        .and_then(|left| right.digest.and_then(|right| rule::pair(left, right)));
    Summary {
        digest: derived.map(|(digest, _)| digest),
        // The dependency capacity bounds ALL logical canonical leaf inputs,
        // not merely the root digest. Pair material is derived work and is
        // reported separately at every actual changed-path derivation.
        content_bytes: left.content_bytes.and_then(|left| {
            right
                .content_bytes
                .and_then(|right| left.checked_add(right))
        }),
        reusable: left.reusable && right.reusable,
    }
}

fn repeated(mut work: Work, part: Work, count: u64) -> Work {
    // Exactly the derives executed by this tree operation, never subtree cost.
    for _ in 0..count {
        work = work.combine(part);
    }
    work
}

impl FactContent {
    pub(in crate::domain_computation::primary_graph) fn build(
        body: FactPayload,
        control: StorageControl<'_, '_>,
    ) -> Result<(Self, Work), StoreDenial> {
        let _canonical = admission::scratch(control)?;
        let mut memory = admit_tree(body.len(), control)?;
        let fields = fields::Fields::build(&body, control)?;
        let (empty, empty_work) = rule::empty().map_err(|()| StoreDenial::ConflictingBody)?;
        let (_, pair_work) = rule::pair(empty, empty).map_err(|()| StoreDenial::ConflictingBody)?;
        let mut work = empty_work.combine(pair_work);
        let mut entries = Vec::with_capacity(body.len());
        for (ordinal, fact) in body.iter().enumerate() {
            let (summary, billed) = leaf(ordinal, fact, control)?;
            entries.push((PartitionIdentity::new(ordinal as u64), summary));
            work = work.combine(billed);
        }
        let plan =
            ReductionPlan::try_from_sorted_unique(entries.iter().map(|(id, _)| *id).collect())
                .map_err(|_| StoreDenial::Representability)?;
        let (tree, metrics) = Tree::try_from_declared_checked(
            plan,
            entries,
            Summary {
                digest: Ok(empty),
                content_bytes: Some(0),
                reusable: true,
            },
            combine as fn(&Summary, &Summary) -> Summary,
            VALUE_BOUND,
            || control.check_live(),
        )
        .map_err(admission::tree_denial)?;
        work = repeated(work, pair_work, metrics.combine_calls);
        admission::settle(&mut memory, &tree)?;
        control.check_live()?;
        Ok((
            Self(Arc::new(Content {
                body,
                tree,
                pair_work,
                fields,
                _memory: memory,
            })),
            work,
        ))
    }

    /// Selection is issued by completed native read custody, not by a caller's
    /// digest. The new body is already admitted and remains a distinct owner.
    pub(in crate::domain_computation::primary_graph) fn refresh(
        &self,
        body: FactPayload,
        changed: impl IntoIterator<Item = usize>,
        control: StorageControl<'_, '_>,
    ) -> Result<(Self, Work), StoreDenial> {
        let _canonical = admission::scratch(control)?;
        let mut memory = admit_tree(self.0.body.len().max(body.len()), control)?;
        let mut tree = self.0.tree.clone();
        let mut addresses_match = true;
        let mut work = Work::zero();
        for ordinal in body.len()..self.0.body.len() {
            addresses_match &= fields::Fields::addresses_match(self.0.body.get(ordinal), None);
            let metrics = tree
                .delete_checked(PartitionIdentity::new(ordinal as u64), VALUE_BOUND, || {
                    control.check_live()
                })
                .map_err(admission::tree_denial)?;
            work = repeated(work, self.0.pair_work, metrics.combine_calls);
        }
        for ordinal in changed {
            let fact = body.get(ordinal).ok_or(StoreDenial::ConflictingBody)?;
            addresses_match &=
                fields::Fields::addresses_match(self.0.body.get(ordinal), Some(fact));
            let (summary, billed) = leaf(ordinal, fact, control)?;
            let identity = PartitionIdentity::new(ordinal as u64);
            let metrics = if ordinal < self.0.body.len() {
                tree.update_checked(identity, summary, VALUE_BOUND, || control.check_live())
            } else {
                tree.insert_checked(identity, summary, VALUE_BOUND, || control.check_live())
            }
            .map_err(admission::tree_denial)?;
            work = repeated(
                work.combine(billed),
                self.0.pair_work,
                metrics.combine_calls,
            );
        }
        if tree.partition_count() != body.len() {
            return Err(StoreDenial::ConflictingBody);
        }
        admission::settle(&mut memory, &tree)?;
        let fields = if addresses_match {
            Arc::clone(&self.0.fields)
        } else {
            fields::Fields::build(&body, control)?
        };
        control.check_live()?;
        Ok((
            Self(Arc::new(Content {
                body,
                tree,
                pair_work: self.0.pair_work,
                fields,
                _memory: memory,
            })),
            work,
        ))
    }

    pub(in crate::domain_computation::primary_graph) fn same_body(
        &self,
        body: &FactPayload,
    ) -> bool {
        self.0.body.same_backing(body)
    }
    pub(super) fn identity(&self, key: [u8; 32], maximum: usize) -> rule::Derived {
        let summary = self.0.tree.result();
        let remaining = maximum
            .checked_sub(summary.content_bytes.ok_or(())?)
            .ok_or(())?;
        rule::root(key, self.0.body.len(), summary.digest?, remaining)
    }
    pub(in crate::domain_computation::primary_graph) fn reusable(&self) -> bool {
        self.0.tree.result().reusable
    }
    pub(in crate::domain_computation::primary_graph) fn charged_bytes(&self) -> Option<u64> {
        self.0
            .tree
            .additional_charged_bytes()
            .checked_add(self.0.fields.bytes())?
            .checked_add((size_of::<Content>() + 2 * size_of::<usize>()) as u64)
    }
}

#[cfg(test)]
pub(super) fn dependency_identity(
    key: [u8; 32],
    facts: &[super::super::WorthQueryApplicationObservedFact],
    maximum: usize,
) -> rule::Derived {
    let body: FactPayload = Arc::<[_]>::from(facts).into();
    let (content, work) = FactContent::build(
        body,
        StorageControl::new(
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            None,
        ),
    )
    .map_err(|_| ())?;
    content
        .identity(key, maximum)
        .map(|(digest, root)| (digest, work.combine(root)))
}
