use std::sync::Arc;

use worth_query_installation::facade::ApplicationSchema;

use super::super::{
    WorthQueryApplicationProducerBinding, WorthQueryInstalledApplicationProducerRegistry,
};

pub struct WorthQueryApplicationConditionalProducerAccess<'a, Schema> {
    installed: &'a WorthQueryInstalledApplicationProducerRegistry<Schema>,
    required: &'a [String],
}

impl<'a, Schema> WorthQueryApplicationConditionalProducerAccess<'a, Schema>
where
    Schema: ApplicationSchema,
{
    pub(super) fn new(
        installed: &'a WorthQueryInstalledApplicationProducerRegistry<Schema>,
        required: &'a [String],
    ) -> Self {
        Self {
            installed,
            required,
        }
    }

    pub fn provider<Binding>(&self) -> Option<Arc<Binding::Provider>>
    where
        Binding: WorthQueryApplicationProducerBinding<Schema>,
    {
        dependency_is_declared(self.required, Binding::IDENTITY)
            .then(|| self.installed.provider::<Binding>())
            .flatten()
    }
}

fn dependency_is_declared(required: &[String], identity: &str) -> bool {
    required.iter().any(|declared| declared == identity)
}

#[cfg(test)]
mod tests {
    use super::dependency_is_declared;

    #[test]
    fn conditional_dependency_access_is_exact() {
        let required = vec!["rectangle.initial".to_owned()];
        assert!(dependency_is_declared(&required, "rectangle.initial"));
        assert!(!dependency_is_declared(&required, "rectangle.preserve"));
        assert!(!dependency_is_declared(&required, "foreign.initial"));
    }
}
