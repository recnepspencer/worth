//! W8E: one stock-posting access fragment declared in `posting` and included
//! by path from sibling action modules. An action whose access is only that
//! fragment, one combining it with inline access, and one composing two
//! overlapping fragments each install and commit, and the fragment grants no
//! access it does not name.
mod court;
mod moves;
mod posting;
mod program;
mod receipts;
mod schema;
mod transfers;
