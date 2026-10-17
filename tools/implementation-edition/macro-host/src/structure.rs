//! Carried structural bounds; invisible delimiters still consume depth/work.
use super::Failure;
use proc_macro::{TokenStream, TokenTree};

const MAX_DEPTH: usize = 128;
const MAX_NODES: usize = 100_000;
const MAX_BYTES: usize = 32 * 1024 * 1024;

#[derive(Clone, Copy, Default)]
pub(super) struct Structure {
    pub(super) depth: usize,
    pub(super) nodes: usize,
    pub(super) bytes: usize,
}
impl Structure {
    pub(super) fn checked(self) -> Result<Self, Failure> {
        if self.depth > MAX_DEPTH || self.nodes > MAX_NODES || self.bytes > MAX_BYTES {
            return Err(Failure::WorkLimit);
        }
        Ok(self)
    }
    pub(super) fn group(self) -> Result<Self, Failure> {
        Self {
            depth: self.depth + 1,
            nodes: self.nodes + 1,
            bytes: self.bytes + 8,
        }
        .checked()
    }
    pub(super) fn concatenate(self, other: Self) -> Result<Self, Failure> {
        Self {
            depth: self.depth.max(other.depth),
            nodes: self.nodes + other.nodes,
            bytes: self.bytes + other.bytes,
        }
        .checked()
    }
    pub(super) fn children(self) -> Self {
        Self {
            depth: self.depth.saturating_sub(1),
            nodes: self.nodes.saturating_sub(1),
            bytes: self.bytes,
        }
    }
}

pub(super) struct Work(usize);
impl Default for Work {
    fn default() -> Self {
        Self(4_000_000)
    }
}
impl Work {
    pub(super) fn charge(&mut self, nodes: usize) -> Result<(), Failure> {
        self.0 = self.0.checked_sub(nodes.max(1)).ok_or(Failure::WorkLimit)?;
        Ok(())
    }
    // Only existing compiler input or bounded text parsing needs measurement.
    // Broker-created groups/streams carry facts instead of rewalking children.
    pub(super) fn measure(&mut self, stream: TokenStream) -> Result<Structure, Failure> {
        let mut frames = vec![(stream.into_iter(), 0)];
        let mut structure = Structure::default();
        while let Some((iter, depth)) = frames.last_mut() {
            let Some(token) = iter.next() else {
                frames.pop();
                continue;
            };
            self.charge(1)?;
            structure.nodes += 1;
            match token {
                TokenTree::Group(group) => {
                    let next_depth = *depth + 1;
                    structure.depth = structure.depth.max(next_depth);
                    structure.bytes += 8;
                    structure.checked()?;
                    frames.push((group.stream().into_iter(), next_depth));
                }
                scalar => {
                    // Scalar rendering has no recursive token structure.
                    structure.bytes += scalar.to_string().len() + 8;
                    structure.checked()?;
                }
            }
        }
        structure.checked()
    }
}
