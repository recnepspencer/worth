use worth_foundational::facade::{
    CanonicalBasisDomain, CanonicalBasisEntry, CanonicalBasisEntryKind, CanonicalBasisLocus,
    CanonicalBasisValue, CanonicalIntegerWidth,
};

pub(super) const APPLICATION_SCHEMA_DOMAIN: CanonicalBasisDomain =
    CanonicalBasisDomain::Future("worth-query.application-schema");

pub(super) struct ApplicationSchemaCanonicalBasis {
    entries: Vec<CanonicalBasisEntry>,
    source_bytes: u64,
    maximum_source_bytes: u64,
    maximum_entries: u64,
    denial: Option<ApplicationSchemaCanonicalBasisBudgetDenial>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct ApplicationSchemaCanonicalBasisWork {
    pub source_bytes: u64,
    pub entries: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ApplicationSchemaCanonicalBasisBudgetDenial {
    SourceBytes { observed: u64, maximum: u64 },
    Entries { observed: u64, maximum: u64 },
}

impl ApplicationSchemaCanonicalBasis {
    pub(super) fn with_capacity_and_limits(
        member_count: usize,
        contribution_entries: usize,
        maximum_source_bytes: u64,
        maximum_entries: u64,
    ) -> Self {
        let requested_capacity = member_count
            .saturating_mul(4)
            .saturating_add(contribution_entries)
            .saturating_add(6);
        let maximum_capacity = usize::try_from(maximum_entries).unwrap_or(usize::MAX);
        Self {
            entries: Vec::with_capacity(requested_capacity.min(maximum_capacity)),
            source_bytes: 0,
            maximum_source_bytes,
            maximum_entries,
            denial: None,
        }
    }

    pub(super) const fn is_denied(&self) -> bool {
        self.denial.is_some()
    }

    pub(super) fn text(&mut self, locus: impl Into<String>, value: &str) {
        let locus = locus.into();
        if self.admit(&locus, u64::try_from(value.len()).unwrap_or(u64::MAX)) {
            self.push_admitted(
                locus,
                CanonicalBasisValue::ExactText(value.to_owned().into()),
            );
        }
    }

    pub(super) fn optional_text(&mut self, locus: impl Into<String>, value: Option<&str>) {
        let locus = locus.into();
        let value_bytes = value.map_or(1, |value| u64::try_from(value.len()).unwrap_or(u64::MAX));
        if self.admit(&locus, value_bytes) {
            self.push_admitted(
                locus,
                value.map_or(CanonicalBasisValue::Null, |value| {
                    CanonicalBasisValue::ExactText(value.to_owned().into())
                }),
            );
        }
    }

    pub(super) fn bool(&mut self, locus: impl Into<String>, value: bool) {
        self.push(locus, CanonicalBasisValue::Bool(value));
    }

    pub(super) fn u32(&mut self, locus: impl Into<String>, value: u32) {
        self.push(
            locus,
            CanonicalBasisValue::UnsignedInteger {
                width: CanonicalIntegerWidth::Bits32,
                value: value.into(),
            },
        );
    }

    pub(super) fn u64(&mut self, locus: impl Into<String>, value: u64) {
        self.push(
            locus,
            CanonicalBasisValue::UnsignedInteger {
                width: CanonicalIntegerWidth::Bits64,
                value: value.into(),
            },
        );
    }

    pub(super) fn usize(&mut self, locus: impl Into<String>, value: usize) {
        self.push(
            locus,
            CanonicalBasisValue::UnsignedInteger {
                width: CanonicalIntegerWidth::Bits64,
                value: value as u128,
            },
        );
    }

    pub(super) fn value(&mut self, locus: impl Into<String>, value: CanonicalBasisValue) {
        self.push(locus, value);
    }

    pub(super) fn into_entries(
        self,
    ) -> Result<
        (
            Vec<CanonicalBasisEntry>,
            ApplicationSchemaCanonicalBasisWork,
        ),
        ApplicationSchemaCanonicalBasisBudgetDenial,
    > {
        match self.denial {
            Some(denial) => Err(denial),
            None => {
                let entries = u64::try_from(self.entries.len()).unwrap_or(u64::MAX);
                Ok((
                    self.entries,
                    ApplicationSchemaCanonicalBasisWork {
                        source_bytes: self.source_bytes,
                        entries,
                    },
                ))
            }
        }
    }

    pub(super) fn extend_embedded(
        &mut self,
        source: &worth_foundational::facade::CanonicalBasisReadyArtifact,
        locus_prefix: &str,
        kind: CanonicalBasisEntryKind,
    ) {
        for entry in source.payload().entries() {
            let name = match entry.locus() {
                CanonicalBasisLocus::Named(worth_foundational::facade::InternedString::Raw(
                    name,
                )) => name,
                _ => unreachable!("application schema embeds named raw canonical loci"),
            };
            let locus = format!("{locus_prefix}.{name}");
            if self.admit(&locus, canonical_value_source_bytes(entry.value())) {
                self.entries.push(CanonicalBasisEntry::new(
                    APPLICATION_SCHEMA_DOMAIN,
                    CanonicalBasisLocus::Named(locus.into()),
                    kind,
                    entry.value().clone(),
                ));
            }
        }
    }

    fn push(&mut self, locus: impl Into<String>, value: CanonicalBasisValue) {
        let locus = locus.into();
        if self.admit(&locus, canonical_value_source_bytes(&value)) {
            self.push_admitted(locus, value);
        }
    }

    fn push_admitted(&mut self, locus: String, value: CanonicalBasisValue) {
        self.entries.push(CanonicalBasisEntry::new(
            APPLICATION_SCHEMA_DOMAIN,
            CanonicalBasisLocus::Named(locus.into()),
            CanonicalBasisEntryKind::Identity,
            value,
        ));
    }

    fn admit(&mut self, locus: &str, value_bytes: u64) -> bool {
        if self.denial.is_some() {
            return false;
        }
        let observed_entries = u64::try_from(self.entries.len())
            .unwrap_or(u64::MAX)
            .saturating_add(1);
        if observed_entries > self.maximum_entries {
            self.denial = Some(ApplicationSchemaCanonicalBasisBudgetDenial::Entries {
                observed: observed_entries,
                maximum: self.maximum_entries,
            });
            return false;
        }
        let observed_bytes = self
            .source_bytes
            .checked_add(u64::try_from(locus.len()).unwrap_or(u64::MAX))
            .and_then(|bytes| bytes.checked_add(value_bytes))
            .unwrap_or(u64::MAX);
        if observed_bytes > self.maximum_source_bytes {
            self.denial = Some(ApplicationSchemaCanonicalBasisBudgetDenial::SourceBytes {
                observed: observed_bytes,
                maximum: self.maximum_source_bytes,
            });
            return false;
        }
        self.source_bytes = observed_bytes;
        true
    }
}

fn canonical_value_source_bytes(value: &CanonicalBasisValue) -> u64 {
    use CanonicalBasisValue as Value;
    match value {
        Value::Null | Value::Bool(_) => 1,
        Value::SignedInteger { .. }
        | Value::UnsignedInteger { .. }
        | Value::FloatBits { .. }
        | Value::BytesRefId(_)
        | Value::ContentRefId(_)
        | Value::DateDays(_)
        | Value::TimeNanos(_)
        | Value::TimestampMicros(_)
        | Value::NestedSequence(_) => 16,
        Value::TimestampTz { .. } | Value::EntityRef { .. } => 24,
        Value::BytesDigest(_) | Value::UuidBytes(_) => 32,
        Value::ExactText(value) | Value::DecimalText(value) | Value::BigIntText(value) => {
            interned_string_bytes(value)
        }
        Value::RationalText {
            numerator,
            denominator,
        } => interned_string_bytes(numerator).saturating_add(interned_string_bytes(denominator)),
    }
}

fn interned_string_bytes(value: &worth_foundational::facade::InternedString) -> u64 {
    match value {
        worth_foundational::facade::InternedString::Raw(value) => {
            u64::try_from(value.len()).unwrap_or(u64::MAX)
        }
        worth_foundational::facade::InternedString::Symbol(_) => 8,
    }
}
