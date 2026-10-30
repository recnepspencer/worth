pub use worth_query_host::facade::domain;

pub struct HostCondition;

impl domain::WorthQueryDomainConditionFamily for HostCondition {
    const PORTABLE_IDENTITY: &'static str = "worth.host.tests.condition";
}
