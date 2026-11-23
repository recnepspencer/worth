//! Tracked, unprivileged persistent-name reads at a reader's pinned basis.
//!
//! [`ExecutionNameReader`] answers `name_of` (the admitted name a live record
//! carries) and `resolve` (where a held name's meaning lives now) from
//! Relational's branch-local derived name index, exactly as the reader's
//! observation sees it. Every answer is kind-checked against the installed
//! scheme's declared subject, so a name of one kind never answers for
//! another, and every answer records what it rests on into
//! [`NameReadDependencies`]. A later commit stales a recorded read only when
//! its canonical lineage events end, replace, split, merge or restore a
//! lineage that read rested on, or birth a name the read found unknown;
//! unrelated commits leave it clean.

mod denial;
mod dependencies;
mod reader;
mod resolve;
mod selection_lowering;

pub use denial::{ExecutionNameReadDenial, ExecutionNameResolveDenial};
pub use dependencies::{NameReadCurrentness, NameReadDependencies, NameReadStaleness};
pub use reader::ExecutionNameReader;
