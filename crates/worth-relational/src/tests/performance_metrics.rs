use std::collections::BTreeMap;
use std::ops::Index;

use serde::ser::{SerializeMap, Serializer};
use serde::Serialize;

use crate::performance::data::RuntimeComplexityCounters;
use crate::query::data::QueryComplexitySummary;

mod serde_struct_projection;

use self::serde_struct_projection::structured_metric_group;

#[derive(Debug, Clone, Default, PartialEq)]
pub(super) struct PerfMetricSet {
    fields: BTreeMap<&'static str, PerfMetricValue>,
}

impl PerfMetricSet {
    pub(super) fn new() -> Self {
        Self::default()
    }

    pub(super) fn insert_value(&mut self, key: &'static str, value: impl Into<PerfMetricValue>) {
        self.fields.insert(key, value.into());
    }

    pub(super) fn metric_u64(&self, key: &str) -> Option<u64> {
        self.fields.get(key).and_then(PerfMetricValue::as_u64)
    }

    pub(super) fn metric_path_u128(&self, path: &[&str]) -> Option<u128> {
        let (last_key, parent_path) = path.split_last()?;
        let parent = parent_path
            .iter()
            .try_fold(self, |current, key| current.group(key))?;
        parent
            .fields
            .get(last_key)
            .and_then(PerfMetricValue::as_u128)
    }

    fn group(&self, key: &str) -> Option<&PerfMetricSet> {
        match self.fields.get(key)? {
            PerfMetricValue::Group(group) => Some(group),
            _ => None,
        }
    }
}

impl Index<&str> for PerfMetricSet {
    type Output = PerfMetricValue;

    fn index(&self, index: &str) -> &Self::Output {
        self.fields
            .get(index)
            .unwrap_or_else(|| panic!("missing performance metric `{index}`"))
    }
}

impl Serialize for PerfMetricSet {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.fields.len()))?;
        for (key, value) in &self.fields {
            map.serialize_entry(key, value)?;
        }
        map.end()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(super) enum PerfMetricValue {
    Null,
    Bool(bool),
    Number(u128),
    SignedNumber(i128),
    Float(f64),
    Text(String),
    Group(PerfMetricSet),
}

impl PerfMetricValue {
    pub(super) fn as_u64(&self) -> Option<u64> {
        match self {
            Self::Number(value) => u64::try_from(*value).ok(),
            Self::SignedNumber(value) => u64::try_from(*value).ok(),
            _ => None,
        }
    }

    pub(super) fn as_u128(&self) -> Option<u128> {
        match self {
            Self::Number(value) => Some(*value),
            Self::SignedNumber(value) => u128::try_from(*value).ok(),
            _ => None,
        }
    }

    pub(super) fn as_str(&self) -> Option<&str> {
        match self {
            Self::Text(value) => Some(value),
            _ => None,
        }
    }

    pub(super) fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(value) => Some(*value),
            _ => None,
        }
    }

    pub(super) fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }
}

impl Index<&str> for PerfMetricValue {
    type Output = PerfMetricValue;

    fn index(&self, index: &str) -> &Self::Output {
        match self {
            Self::Group(group) => &group[index],
            _ => panic!("performance metric `{index}` requested from non-group metric"),
        }
    }
}

impl Serialize for PerfMetricValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Null => serializer.serialize_none(),
            Self::Bool(value) => serializer.serialize_bool(*value),
            Self::Number(value) => serializer.serialize_u128(*value),
            Self::SignedNumber(value) => serializer.serialize_i128(*value),
            Self::Float(value) => serializer.serialize_f64(*value),
            Self::Text(value) => serializer.serialize_str(value),
            Self::Group(group) => group.serialize(serializer),
        }
    }
}

impl From<PerfMetricSet> for PerfMetricValue {
    fn from(value: PerfMetricSet) -> Self {
        Self::Group(value)
    }
}

impl From<bool> for PerfMetricValue {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<String> for PerfMetricValue {
    fn from(value: String) -> Self {
        Self::Text(value)
    }
}

impl From<&str> for PerfMetricValue {
    fn from(value: &str) -> Self {
        Self::Text(value.to_string())
    }
}

impl From<Option<String>> for PerfMetricValue {
    fn from(value: Option<String>) -> Self {
        value.map(Self::Text).unwrap_or(Self::Null)
    }
}

impl From<usize> for PerfMetricValue {
    fn from(value: usize) -> Self {
        Self::Number(value as u128)
    }
}

impl From<u64> for PerfMetricValue {
    fn from(value: u64) -> Self {
        Self::Number(u128::from(value))
    }
}

impl From<u32> for PerfMetricValue {
    fn from(value: u32) -> Self {
        Self::Number(u128::from(value))
    }
}

impl From<u128> for PerfMetricValue {
    fn from(value: u128) -> Self {
        Self::Number(value)
    }
}

impl From<i128> for PerfMetricValue {
    fn from(value: i128) -> Self {
        Self::SignedNumber(value)
    }
}

impl From<i32> for PerfMetricValue {
    fn from(value: i32) -> Self {
        Self::SignedNumber(i128::from(value))
    }
}

impl From<f64> for PerfMetricValue {
    fn from(value: f64) -> Self {
        Self::Float(value)
    }
}

impl From<QueryComplexitySummary> for PerfMetricValue {
    fn from(complexity: QueryComplexitySummary) -> Self {
        Self::Group(structured_metric_group(&complexity))
    }
}

impl From<RuntimeComplexityCounters> for PerfMetricValue {
    fn from(counters: RuntimeComplexityCounters) -> Self {
        Self::Group(structured_metric_group(&counters))
    }
}

macro_rules! perf_metrics {
    ({ $($entries:tt)* }) => {{
        let mut metrics = $crate::tests::performance_metrics::PerfMetricSet::new();
        $crate::tests::performance_metrics::perf_metric_entries!(metrics, $($entries)*);
        metrics
    }};
}

macro_rules! perf_metric_entries {
    ($metrics:ident,) => {};
    ($metrics:ident) => {};
    ($metrics:ident, $key:literal : { $($nested:tt)* } $(, $($rest:tt)*)?) => {{
        $metrics.insert_value($key, $crate::tests::performance_metrics::perf_metrics!({ $($nested)* }));
        $crate::tests::performance_metrics::perf_metric_entries!($metrics $(, $($rest)*)?);
    }};
    ($metrics:ident, $key:literal : $value:expr $(, $($rest:tt)*)?) => {{
        $metrics.insert_value($key, $value);
        $crate::tests::performance_metrics::perf_metric_entries!($metrics $(, $($rest)*)?);
    }};
}

pub(super) use perf_metric_entries;
pub(super) use perf_metrics;
