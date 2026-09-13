use std::path::PathBuf;

use crate::product::{CiTestLane, TestProduct};

use super::Arguments;

pub(super) fn parse(arguments: impl IntoIterator<Item = String>) -> Result<Arguments, String> {
    let parsed = ParsedArguments::collect(arguments)?;
    let product = parsed.product()?;
    parsed.validate(&product)?;
    Ok(parsed.finish(product))
}

struct ParsedArguments {
    command: String,
    package: Option<String>,
    partition: Option<CiTestLane>,
    shard_index: Option<usize>,
    shard_count: Option<usize>,
    list: bool,
    target_root: Option<PathBuf>,
}

impl ParsedArguments {
    fn collect(arguments: impl IntoIterator<Item = String>) -> Result<Self, String> {
        let mut arguments = arguments.into_iter();
        let command = arguments.next().ok_or_else(super::usage)?;
        let mut parsed = Self {
            command,
            package: None,
            partition: None,
            shard_index: None,
            shard_count: None,
            list: false,
            target_root: None,
        };
        while let Some(option) = arguments.next() {
            parsed.accept_option(option, &mut arguments)?;
        }
        Ok(parsed)
    }

    fn accept_option(
        &mut self,
        option: String,
        arguments: &mut impl Iterator<Item = String>,
    ) -> Result<(), String> {
        match option.as_str() {
            "-p" | "--package" => self.package = Some(value(arguments, &option)?),
            "--partition" => {
                self.partition = Some(value(arguments, &option)?.parse::<CiTestLane>()?)
            }
            "--shard-index" => self.shard_index = Some(number(arguments, &option)?),
            "--shard-count" => self.shard_count = Some(number(arguments, &option)?),
            "--target-root" => self.target_root = Some(path(arguments, &option)?),
            "--list" => self.list = true,
            unknown => return Err(format!("unknown argument `{unknown}`\n{}", super::usage())),
        }
        Ok(())
    }

    fn product(&self) -> Result<TestProduct, String> {
        match self.command.as_str() {
            "owner" => Ok(TestProduct::Owner {
                package: self
                    .package
                    .clone()
                    .ok_or_else(|| "owner requires -p <package>".to_owned())?,
            }),
            "smoke" => Ok(TestProduct::Smoke),
            "ui" => Ok(TestProduct::Ui),
            "ci" => Ok(TestProduct::Ci {
                lane: self
                    .partition
                    .ok_or_else(|| "ci requires --partition <lane>".to_owned())?,
                shard: shard(self.shard_index, self.shard_count)?,
            }),
            unknown => Err(format!("unknown command `{unknown}`\n{}", super::usage())),
        }
    }

    fn validate(&self, product: &TestProduct) -> Result<(), String> {
        if !matches!(product, TestProduct::Owner { .. }) && self.package.is_some() {
            return Err("-p/--package is valid only for owner".into());
        }
        if !matches!(product, TestProduct::Ci { .. })
            && (self.partition.is_some()
                || self.shard_index.is_some()
                || self.shard_count.is_some())
        {
            return Err("partition and shard arguments are valid only for ci".into());
        }
        Ok(())
    }

    fn finish(self, product: TestProduct) -> Arguments {
        Arguments {
            product,
            list: self.list,
            target_root: self.target_root,
        }
    }
}

fn value(arguments: &mut impl Iterator<Item = String>, option: &str) -> Result<String, String> {
    arguments
        .next()
        .ok_or_else(|| format!("{option} requires a value"))
}

fn number(arguments: &mut impl Iterator<Item = String>, option: &str) -> Result<usize, String> {
    let raw = value(arguments, option)?;
    raw.parse()
        .map_err(|_| format!("{option} requires a non-negative integer, got `{raw}`"))
}

fn path(arguments: &mut impl Iterator<Item = String>, option: &str) -> Result<PathBuf, String> {
    value(arguments, option).map(PathBuf::from)
}

fn shard(index: Option<usize>, count: Option<usize>) -> Result<Option<(usize, usize)>, String> {
    match (index, count) {
        (None, None) => Ok(None),
        (Some(index), Some(count)) if count > 0 && index < count => Ok(Some((index, count))),
        (Some(_), Some(0)) => Err("--shard-count must be greater than zero".into()),
        (Some(index), Some(count)) => Err(format!(
            "--shard-index {index} is outside shard count {count}"
        )),
        _ => Err("--shard-index and --shard-count must be supplied together".into()),
    }
}
