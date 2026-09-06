pub mod product;
use super::observation::SupplyChainObservation;

/// No runtime vocabulary or implementation helpers enter the expected world.
#[derive(Clone, Debug)]
pub struct CompositeWorldOracle {
    pub records: SupplyChainObservation,
    pub occurrences: usize,
}
impl CompositeWorldOracle {
    pub fn bootstrap() -> Self {
        Self {
            records: SupplyChainObservation {
                records: [
                    ("harbor", "open"),
                    ("voyage", "12"),
                    ("manifest", "harbor"),
                    ("grain", "4"),
                    ("steel", "6"),
                ]
                .into_iter()
                .map(|(k, v)| (k.into(), v.into()))
                .collect(),
                links: [
                    ("voyage", "manifest"),
                    ("manifest", "grain"),
                    ("manifest", "steel"),
                    ("manifest", "harbor"),
                ]
                .into_iter()
                .map(|(a, b)| (a.into(), b.into()))
                .collect(),
            },
            occurrences: 1,
        }
    }
    pub fn change(&mut self, cargo: &str, amount: &str) {
        self.records.records.insert(cargo.into(), amount.into());
        self.occurrences += 1;
    }
    pub fn route(&self) -> u64 {
        if self.records.records["harbor"] != "open" || self.records.records["manifest"] != "harbor"
        {
            return 0;
        }
        let mut remaining: u64 = self.records.records["voyage"].parse().unwrap();
        let mut delivered = 0;
        for cargo in ["grain", "steel"] {
            if !self
                .records
                .links
                .contains(&("manifest".into(), cargo.into()))
            {
                continue;
            }
            let tonnes = self.records.records[cargo].parse::<u64>().unwrap();
            let Some(next) = remaining.checked_sub(tonnes) else {
                return 0;
            };
            remaining = next;
            delivered += tonnes;
        }
        delivered
    }
}
