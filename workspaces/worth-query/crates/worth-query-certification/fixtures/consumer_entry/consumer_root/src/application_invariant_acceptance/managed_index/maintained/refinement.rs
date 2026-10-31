//! Explicit exact access shares the maintained Source and candidate provider.
use super::{prefix, query as source, schema::*};
use worth_query_decl::facade::{application_query::*, application_schema::*, *};
mod query;
pub(super) mod rule;
pub(super) use query::*;
