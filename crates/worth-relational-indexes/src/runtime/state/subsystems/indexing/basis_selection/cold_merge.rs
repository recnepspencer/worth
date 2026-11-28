//! Deferred image selection fills missing bindings without publishing old generations.
use super::{Arc, BasisSelections};

impl BasisSelections {
    pub(in crate::internal_port::runtime::state::subsystems::indexing) fn merge(
        &mut self,
        previous: &Self,
    ) {
        for (key, restored) in &previous.entries {
            let basis = match self.entries.get(key) {
                None => Arc::clone(restored),
                Some(live) => {
                    if live.commit != restored.commit || live.schema != restored.schema {
                        continue;
                    }
                    let mut combined = live.as_ref().clone();
                    let state = Arc::make_mut(&mut combined.state);
                    state.fill_missing(&restored.state);
                    Arc::new(combined)
                }
            };
            // Each branch/global lookup is independent; installation of a branch
            // would overwrite a global selection that may already be live.
            self.replace_lookup(key.clone(), Some(basis));
        }
    }
}

#[cfg(test)]
pub mod tests;
