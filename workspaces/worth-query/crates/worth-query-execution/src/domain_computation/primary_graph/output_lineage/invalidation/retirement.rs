//! Superseded settlement rows leave the live mark root once nothing reads them.
//!
//! Retained source positions keep their own immutable roots, so only the live
//! root changes. A row is reachable while a live row consumes it, and an
//! equality reader follows the tail. Each certified successor shares the output
//! projection's custody, so an unread predecessor need not keep obsolete inputs.

use std::sync::Arc;

use worth_relational::facade::mvcc::CompanionBranchCell;
use worth_relational::facade::mvcc::CompanionPreflightStop;

use super::super::RecordedSettlementIdentity;

use super::derived::PreparedSettlementRegistration;

use super::mark_state::EqualOutputLink;
use super::mark_state::MarkState;
use super::settlement;
use super::source_alignment::BranchMarkRoot;
use super::SettlementRegistrationStop;
use super::SourceInvalidationOwner;

type Identity = Arc<RecordedSettlementIdentity>;

impl SourceInvalidationOwner {
    /// Marks each candidate superseded, then retires every superseded row that
    /// nothing live reaches, following the upstream rows it held. Returns the
    /// identities no branch retains any more. Every branch edit is admitted
    /// before any installs, so an admission stop retires nothing; a later
    /// supersession or cleanup retries it.
    pub(in crate::domain_computation::primary_graph) fn retire_settlements(
        &self,
        candidates: &[Identity],
    ) -> Vec<Identity> {
        self.retire_settlements_admitted(candidates)
            .unwrap_or_default()
    }

    /// Retires rows whose owner released them: a demand record that ended, or
    /// a publication whose record displaced their generation. No index names
    /// a released row again, so the ones a live row still reads, or an
    /// admission stop left in place, wait here for the next release. Returns
    /// the identities no branch retains any more.
    pub(in crate::domain_computation::primary_graph) fn retire_released(
        &self,
        released: impl IntoIterator<Item = Identity>,
    ) -> Vec<Identity> {
        let waiting = || {
            self.released
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
        };
        let mut candidates = std::mem::take(&mut *waiting());
        candidates.extend(released);
        let retired = self.retire_settlements(&candidates);
        candidates.retain(|identity| !retired.contains(identity));
        waiting().append(&mut candidates);
        retired
    }

    fn retire_settlements_admitted(
        &self,
        candidates: &[Identity],
    ) -> Result<Vec<Identity>, SettlementRegistrationStop> {
        if candidates.is_empty() {
            return Ok(Vec::new());
        }
        let cells = {
            let branches = self
                .branches
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);

            branches
                .cells
                .values()
                .filter_map(|slot| slot.admitted())
                .collect::<Vec<_>>()
        };

        let mut visited = candidates.to_vec();

        let mut edits = Vec::with_capacity(cells.len());
        for cell in &cells {
            edits.push(self.retire_in_cell(cell.clone(), candidates, &mut visited)?);
        }
        // Retired means retained by no live branch root once the edits
        // install, including as the upstream of a row that is still live.
        let mut retired = Vec::new();

        for identity in visited {
            let mut held = false;
            for (state, _) in &edits {
                held |= state.settlements.contains_key(&identity)
                    || state.downstream.contains_key(&identity);
            }
            if !held && !retired.contains(&identity) {
                retired.push(identity);
            }
        }
        for (_, prepared) in edits {
            let Some(prepared) = prepared else {
                continue;
            };
            let cleanup = prepared
                .install()
                .map_err(|stopped| SettlementRegistrationStop::Edit(stopped.reason()))?;
            drop(cleanup);
        }
        Ok(retired)
    }

    /// The cell's live state once its retirements install, and the prepared
    /// edit that installs them, if any row leaves.
    fn retire_in_cell(
        &self,
        cell: CompanionBranchCell<BranchMarkRoot>,
        candidates: &[Identity],
        visited: &mut Vec<Identity>,
    ) -> Result<(Arc<MarkState>, Option<PreparedSettlementRegistration>), SettlementRegistrationStop>
    {
        let image = cell.read_image();

        let mut state = (*image.payload().current).clone();
        let mut changed = false;
        let mut work: Vec<(Identity, bool)> = Vec::new();

        work.extend(
            candidates
                .iter()
                .map(|identity| (Arc::clone(identity), true)),
        );
        while let Some((identity, requested)) = work.pop() {
            if requested {
                changed |= mark_superseded(&mut state, &identity)?;
            }
            let leaving = match retirable(&state, &identity)? {
                Retirable::No => continue,
                Retirable::Row => {
                    splice_equality(&mut state, &identity)?;
                    vec![identity]
                }
                Retirable::Chain(members) => {
                    for member in &members {
                        state.equal_links.remove(member);
                    }
                    members
                }
            };
            for identity in leaving {
                let row = state
                    .settlements
                    .get(&identity)
                    .cloned()
                    .expect("a retirable row is live");
                settlement::remove(&mut state, &identity)?;
                changed = true;

                visited.push(identity);

                for upstream in &row.consumed_upstream {
                    work.push((Arc::clone(upstream), false));
                }
            }
        }
        if !changed {
            return Ok((Arc::clone(&image.payload().current), None));
        }
        let root = {
            let mut root = (**image.payload()).clone();
            root.current = Arc::new(state);
            root
        };
        let state = Arc::clone(&root.current);
        let prepared = self.prepare_root_replacement(cell, image, Arc::new(root))?;
        Ok((state, Some(prepared)))
    }
}

/// Returns whether the live row changed.
fn mark_superseded(
    state: &mut MarkState,
    identity: &Identity,
) -> Result<bool, CompanionPreflightStop> {
    let Some(row) = state
        .settlements
        .get(identity)
        .filter(|row| !row.superseded)
    else {
        return Ok(false);
    };

    let mut row = (**row).clone();
    row.superseded = true;

    state
        .settlements
        .insert(Arc::clone(identity), Arc::new(row));
    Ok(true)
}

enum Retirable {
    No,
    /// A row outside any equality chain, or an alias between two others.
    Row,
    /// A whole equality chain that no live row reads, head first.
    Chain(Vec<Identity>),
}

/// A superseded row leaves once no live consumer reads it. Readers of any
/// equality member read the chain's tail. A successor carrying the certified
/// output projection lets an unread predecessor splice out; no output proof or
/// its custody is lost. A chain without such a successor leaves together.
fn retirable(state: &MarkState, identity: &Identity) -> Result<Retirable, CompanionPreflightStop> {
    if !unread(state, identity)? {
        return Ok(Retirable::No);
    }

    let Some(link) = state.equal_links.get(identity) else {
        return Ok(Retirable::Row);
    };
    if link.next.as_ref().is_some_and(|next| {
        state
            .settlements
            .get(next)
            .is_some_and(|row| row.output_facts.is_some())
    }) {
        return Ok(Retirable::Row);
    }
    let mut head = Arc::clone(identity);
    let mut found = false;
    for _ in 0..=state.equal_links.len() {
        match state
            .equal_links
            .get(&head)
            .and_then(|link| link.prior.clone())
        {
            Some(prior) => head = prior,
            None => {
                found = true;
                break;
            }
        }
    }
    if !found {
        return Ok(Retirable::No);
    }
    let mut members = Vec::new();
    let mut member = Some(head);
    while let Some(current) = member {
        if members.len() > state.equal_links.len() || !unread(state, &current)? {
            return Ok(Retirable::No);
        }

        member = state
            .equal_links
            .get(&current)
            .and_then(|link| link.next.clone());
        members.push(current);
    }
    Ok(Retirable::Chain(members))
}

fn unread(state: &MarkState, identity: &Identity) -> Result<bool, CompanionPreflightStop> {
    if !state
        .settlements
        .get(identity)
        .is_some_and(|row| row.superseded)
    {
        return Ok(false);
    }

    Ok(state
        .downstream
        .get(identity)
        .is_none_or(|consumers| consumers.is_empty()))
}

/// Readers walk `next` and aliases inherit through `prior`, so a middle alias
/// leaves by joining its two neighbors.
fn splice_equality(
    state: &mut MarkState,
    identity: &Identity,
) -> Result<(), CompanionPreflightStop> {
    let Some(link) = state.equal_links.get(identity).cloned() else {
        return Ok(());
    };
    if let Some(prior) = &link.prior {
        let next = link.next.clone();
        relink(state, prior, |link| link.next = next)?;
    }
    if let Some(next) = &link.next {
        let prior = link.prior.clone();
        relink(state, next, |link| link.prior = prior)?;
    }

    state.equal_links.remove(identity);
    Ok(())
}

/// A link left with neither neighbor carries no equality and is removed.
fn relink(
    state: &mut MarkState,
    identity: &Identity,
    edit: impl FnOnce(&mut EqualOutputLink),
) -> Result<(), CompanionPreflightStop> {
    let Some(link) = state.equal_links.get(identity) else {
        return Ok(());
    };
    let mut link = (**link).clone();
    edit(&mut link);
    if link.prior.is_none() && link.next.is_none() {
        state.equal_links.remove(identity);
        return Ok(());
    }

    state
        .equal_links
        .insert(Arc::clone(identity), Arc::new(link));
    Ok(())
}
