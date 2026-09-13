use std::collections::BTreeMap;
use std::path::Path;

const SCHEMA: &str = "worth.store.physical-work-courtroom.bounded-residency.configuration.v3";
const FIXED_FIELDS: [(&str, u64); 25] = [
    ("inline-record-bytes", 3_000),
    ("inline-records", 64),
    ("extent-record-bytes", 1_048_576),
    ("extent-records", 109),
    ("total-bytes", 6_979_584),
    ("resident-bytes", 65_536),
    ("metadata-bytes", 32_768),
    ("frame-entries", 12),
    ("resident-frames", 8),
    ("pinned-frames", 4),
    ("pin-leases", 6),
    ("dirty-frames", 2),
    ("dirty-replacement-bytes", 65_536),
    ("operation-bytes", 6_815_744),
    ("checkpoint-memory-bytes", 1_048_576),
    ("scope-foreground-read-bytes", 2_097_152),
    ("scope-foreground-write-bytes", 6_815_744),
    ("scope-recovery-bytes", 2_359_296),
    ("scope-scrub-bytes", 1_835_008),
    ("scope-maintenance-bytes", 1_572_864),
    ("scope-verification-bytes", 1_048_576),
    ("scope-blob-bytes", 1_310_720),
    ("speculative-prefetch-frames", 2),
    ("speculative-read-ahead-frames", 2),
    ("speculative-write-behind-frames", 1),
];

#[derive(Debug, Clone, Copy)]
pub(super) struct BoundedResidencyConfiguration {
    seed: u64,
    inline_record_bytes: usize,
    inline_records: usize,
    extent_record_bytes: usize,
    extent_records: usize,
}

impl BoundedResidencyConfiguration {
    pub(super) fn read(path: &Path) -> Result<Self, String> {
        let encoded = std::fs::read_to_string(path)
            .map_err(|error| format!("cannot read bounded-residency configuration: {error}"))?;
        let mut lines = encoded.lines();
        if lines.next() != Some(SCHEMA) {
            return Err("unsupported bounded-residency configuration schema".to_owned());
        }
        let mut fields = BTreeMap::new();
        for line in lines {
            let (name, encoded) = line
                .split_once('=')
                .ok_or_else(|| format!("malformed configuration field `{line}`"))?;
            let value = encoded
                .parse::<u64>()
                .map_err(|_| format!("configuration field `{name}` is not a number"))?;
            if fields.insert(name, value).is_some() {
                return Err(format!("configuration field `{name}` is duplicated"));
            }
        }
        let seed = fields
            .remove("seed")
            .ok_or_else(|| "configuration omitted `seed`".to_owned())?;
        if seed == 0 {
            return Err("configuration seed cannot be zero".to_owned());
        }
        for (name, expected) in FIXED_FIELDS {
            let actual = fields
                .remove(name)
                .ok_or_else(|| format!("configuration omitted `{name}`"))?;
            if actual != expected {
                return Err(format!(
                    "configuration field `{name}` expected {expected}, found {actual}"
                ));
            }
        }
        if !fields.is_empty() {
            return Err(format!(
                "configuration contains undeclared fields: {:?}",
                fields.keys().collect::<Vec<_>>()
            ));
        }
        Ok(Self {
            seed,
            inline_record_bytes: 3_000,
            inline_records: 64,
            extent_record_bytes: 1_048_576,
            extent_records: 109,
        })
    }

    pub(super) const fn seed(self) -> u64 {
        self.seed
    }

    pub(super) const fn record_count(self) -> usize {
        self.inline_records + self.extent_records
    }

    pub(super) const fn record_bytes(self, ordinal: usize) -> Option<usize> {
        if ordinal < self.inline_records {
            Some(self.inline_record_bytes)
        } else if ordinal < self.record_count() {
            Some(self.extent_record_bytes)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::BoundedResidencyConfiguration;

    const CURRENT_PROFILE: &str = "worth.store.physical-work-courtroom.bounded-residency.configuration.v3\nseed=7312955904608109267\ninline-record-bytes=3000\ninline-records=64\nextent-record-bytes=1048576\nextent-records=109\ntotal-bytes=6979584\nresident-bytes=65536\nmetadata-bytes=32768\nframe-entries=12\nresident-frames=8\npinned-frames=4\npin-leases=6\ndirty-frames=2\ndirty-replacement-bytes=65536\noperation-bytes=6815744\ncheckpoint-memory-bytes=1048576\nscope-foreground-read-bytes=2097152\nscope-foreground-write-bytes=6815744\nscope-recovery-bytes=2359296\nscope-scrub-bytes=1835008\nscope-maintenance-bytes=1572864\nscope-verification-bytes=1048576\nscope-blob-bytes=1310720\nspeculative-prefetch-frames=2\nspeculative-read-ahead-frames=2\nspeculative-write-behind-frames=1\n";

    #[test]
    fn current_hostile_profile_is_accepted_independently() {
        let file = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(file.path(), CURRENT_PROFILE).unwrap();
        let configuration =
            BoundedResidencyConfiguration::read(file.path()).unwrap_or_else(|failure| {
                panic!("MUTANT_PREDICATE:c7-offline-bounded-schema-v3-rejected {failure}");
            });
        assert_eq!(configuration.record_count(), 173);
    }

    #[test]
    fn stale_and_impossible_profiles_are_rejected() {
        let file = tempfile::NamedTempFile::new().unwrap();
        for stale in [
            CURRENT_PROFILE.replace("extent-records=109", "extent-records=72"),
            CURRENT_PROFILE.replace("extent-records=109", "extent-records=108"),
            CURRENT_PROFILE.replace("total-bytes=6979584", "total-bytes=6815744"),
            CURRENT_PROFILE.replace("operation-bytes=6815744", "operation-bytes=4194304"),
            CURRENT_PROFILE.replace(
                "checkpoint-memory-bytes=1048576",
                "checkpoint-memory-bytes=16777216",
            ),
        ] {
            std::fs::write(file.path(), stale).unwrap();
            assert!(BoundedResidencyConfiguration::read(file.path()).is_err());
        }
    }

    #[test]
    fn foreign_successor_scope_limits_are_rejected_independently() {
        let file = tempfile::NamedTempFile::new().unwrap();
        let swapped = CURRENT_PROFILE
            .replace(
                "scope-recovery-bytes=2359296",
                "scope-recovery-bytes=1835008",
            )
            .replace("scope-scrub-bytes=1835008", "scope-scrub-bytes=2359296");
        std::fs::write(file.path(), swapped).unwrap();
        let denial = match BoundedResidencyConfiguration::read(file.path()) {
            Ok(_) => panic!("MUTANT_PREDICATE:offline-foreign-successor-scope-accepted"),
            Err(denial) => denial,
        };
        assert!(denial.contains(
            "configuration field `scope-recovery-bytes` expected 2359296, found 1835008"
        ));
    }
}
