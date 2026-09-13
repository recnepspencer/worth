use std::path::PathBuf;

use crate::product::TestProduct;

mod parsing;

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct Arguments {
    pub(crate) product: TestProduct,
    pub(crate) list: bool,
    pub(crate) target_root: Option<PathBuf>,
}

impl Arguments {
    pub(crate) fn parse(arguments: impl IntoIterator<Item = String>) -> Result<Self, String> {
        parsing::parse(arguments)
    }
}

pub(super) fn help_requested(arguments: &[String]) -> bool {
    matches!(arguments, [argument] if argument == "-h" || argument == "--help")
}

pub(super) fn usage() -> String {
    "usage: store-test-runner <owner -p PACKAGE|smoke|ui|ci --partition LANE> \
     [--shard-index N --shard-count N] \
     [--list] [--target-root PATH]"
        .into()
}

#[cfg(test)]
mod tests;
