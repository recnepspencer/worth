//! A small task board declared through `WorthQuerySchema`, `WorthQueryRecord`,
//! `WorthQueryAction` and `WorthQueryProjection`, mixed with hand-written
//! members, installed under bounded serial limits and driven by a
//! local-process principal through source custody.
mod actions;
mod program;
mod reads;
mod relations;
mod schema;

mod journey;
mod parity;
