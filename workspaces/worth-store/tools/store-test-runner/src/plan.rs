use std::collections::BTreeSet;
use std::path::Path;

use crate::product::{CiTestLane, TestProduct};

mod execution_unit;
mod integration_product;
mod offline_observer_build;
mod owner_product;
mod process_scenario_product;
mod smoke_product;
mod structural_product;

use execution_unit::apply_ci_profiles;
pub(crate) use execution_unit::TestExecutionUnit;
use integration_product::{formal, scenario, ui};
use owner_product::{owner, owner_ci};
use process_scenario_product::process_scenario;
use smoke_product::smoke;
use structural_product::structural;

#[derive(Debug, Clone)]
pub(crate) struct TestPlan {
    product: TestProduct,
    units: Vec<TestExecutionUnit>,
}

impl TestPlan {
    pub(crate) fn build(product: &TestProduct, workspace_root: &Path) -> Result<Self, String> {
        let mut units = match product {
            TestProduct::Owner { package } => owner(package, workspace_root),
            TestProduct::Smoke => smoke(workspace_root),
            TestProduct::Ui => ui(None, workspace_root),
            TestProduct::Ci {
                lane: selected,
                shard,
            } => match selected {
                CiTestLane::OwnerUnit if shard.is_none() => owner_ci(workspace_root),
                CiTestLane::Structural if shard.is_none() => structural(workspace_root)?,
                CiTestLane::ProcessScenario if shard.is_none() => process_scenario(workspace_root),
                CiTestLane::OwnerUnit | CiTestLane::ProcessScenario | CiTestLane::Structural => {
                    return Err(format!("the {selected} partition is not shardable"));
                }
                CiTestLane::Scenario => scenario(*shard, workspace_root),
                CiTestLane::Ui => ui(*shard, workspace_root),
                CiTestLane::Formal => formal(*shard, workspace_root),
            },
        };
        if matches!(product, TestProduct::Ci { .. }) {
            apply_ci_profiles(&mut units);
        }
        if units.is_empty() {
            return Err(format!(
                "test product `{}` selected no commands",
                product.name()
            ));
        }
        reject_duplicate_units(product, &units)?;
        Ok(Self {
            product: product.clone(),
            units,
        })
    }

    pub(crate) fn product_name(&self) -> String {
        self.product.name()
    }

    pub(crate) fn units(&self) -> &[TestExecutionUnit] {
        &self.units
    }
}

fn reject_duplicate_units(
    product: &TestProduct,
    units: &[TestExecutionUnit],
) -> Result<(), String> {
    let mut identities = BTreeSet::new();
    for unit in units {
        if !identities.insert(unit.identity()) {
            return Err(format!(
                "test product `{}` repeats execution unit `{}`",
                product.name(),
                unit.identity()
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
