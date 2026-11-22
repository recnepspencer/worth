//! Parameterized residual filtering of ordinary application queries through
//! the public Host: a small task board with its own schema and program.
mod edit;
mod observe;
mod queries;
mod schema;
mod world;

mod admission;
mod combined;
mod interruption;
mod invalidation;
mod ranges;
mod text_search;

use queries::{ByStatus, DueBy, Fit, Search};
use world::{board, install, keys};

fn search_text(project: u64, q: &str) -> Search {
    Search {
        project,
        q: q.to_owned(),
    }
}
