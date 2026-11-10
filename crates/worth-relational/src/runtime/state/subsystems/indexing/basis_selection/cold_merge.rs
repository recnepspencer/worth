//! Deferred image selection fills missing bindings without publishing old generations.
use super::{Arc, BasisSelections};

impl BasisSelections {
    pub(in crate::runtime::state::subsystems::indexing) fn merge(&mut self, previous: &Self) {
        for (key, restored) in &previous.entries {
            let basis = match self.entries.get(key) {
                None => Arc::clone(restored),
                Some(live) => {
                    if live.commit != restored.commit || live.schema != restored.schema {
                        continue;
                    }
                    let mut combined = live.as_ref().clone();
                    let state = Arc::make_mut(&mut combined.state);
                    for (index, generation) in &restored.state.entries {
                        if !state.entries.contains_key(index) && !state.empty.contains(index) {
                            state.entries.insert(*index, Arc::clone(generation));
                            if restored.state.global.contains_key(index) {
                                state.global.insert(*index, Arc::clone(generation));
                            }
                        }
                    }
                    for index in &restored.state.empty {
                        if !state.entries.contains_key(index) && !state.empty.contains(index) {
                            state.empty.insert(*index);
                            if restored.state.global_empty.contains(index) {
                                state.global_empty.insert(*index);
                            }
                        }
                    }
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
mod tests;
