//! Immutable cancellation carriage; repeated original signals need no heap list.
use crate::CancellationToken;
use std::{ops::Index, sync::Arc};

#[derive(Clone, Default)]
pub(super) struct CancellationLineage(Representation);

#[derive(Clone, Default)]
enum Representation {
    #[default]
    Empty,
    Repeated(CancellationToken, usize),
    Shared(Arc<[CancellationToken]>),
}

// Scope memory admission keeps its original framework layout on every target.
const _: () = assert!(size_of::<CancellationLineage>() == size_of::<Vec<CancellationToken>>());
const _: () = assert!(align_of::<CancellationLineage>() == align_of::<Vec<CancellationToken>>());

impl CancellationLineage {
    pub(super) fn len(&self) -> usize {
        match &self.0 {
            Representation::Empty => 0,
            Representation::Repeated(_, count) => *count,
            Representation::Shared(tokens) => tokens.len(),
        }
    }

    pub(super) fn iter(&self) -> impl Iterator<Item = &CancellationToken> {
        (0..self.len()).map(|index| &self[index])
    }

    pub(super) fn push(&mut self, token: CancellationToken) {
        self.extend(&mut std::iter::once(token), 1);
    }

    pub(super) fn extend(
        &mut self,
        tokens: &mut dyn Iterator<Item = CancellationToken>,
        additional: usize,
    ) {
        let capacity = self
            .len()
            .checked_add(additional)
            .expect("checked token count");
        let mut materialized: Option<Vec<CancellationToken>> = None;
        for token in tokens {
            if let Some(tokens) = &mut materialized {
                tokens.push(token);
                continue;
            }
            let count = self.len().checked_add(1).expect("checked token count");
            match &mut self.0 {
                Representation::Empty => self.0 = Representation::Repeated(token, count),
                Representation::Repeated(original, repeated) if original.same_signal(&token) => {
                    *repeated = count;
                }
                _ => {
                    #[cfg(test)]
                    super::cost::materialized();
                    let mut tokens = Vec::with_capacity(capacity);
                    tokens.extend(self.iter().cloned());
                    tokens.push(token);
                    materialized = Some(tokens);
                }
            }
        }
        if let Some(tokens) = materialized {
            self.0 = Representation::Shared(tokens.into());
        }
    }
}

impl Index<usize> for CancellationLineage {
    type Output = CancellationToken;
    fn index(&self, index: usize) -> &Self::Output {
        match &self.0 {
            Representation::Empty => panic!("empty cancellation lineage"),
            Representation::Repeated(token, count) => {
                assert!(index < *count, "cancellation lineage index out of bounds");
                token
            }
            Representation::Shared(tokens) => &tokens[index],
        }
    }
}

#[cfg(test)]
mod tests;
