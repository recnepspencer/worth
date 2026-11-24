#[path = "declarations.rs"]
mod declarations;
pub(crate) use self::declarations::*;

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
