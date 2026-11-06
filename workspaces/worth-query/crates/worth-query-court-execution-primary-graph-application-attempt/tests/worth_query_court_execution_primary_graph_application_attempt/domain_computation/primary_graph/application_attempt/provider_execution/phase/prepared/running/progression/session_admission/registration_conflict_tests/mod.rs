include!("registration_conflict_tests.rs");
// A child, so the court reuses these private helpers without widening them.

#[path = "interrupted_registration_tests/mod.rs"]
pub mod interrupted_registration_tests;
