use crate::application_query::{
    ApplicationQueryBinding, ApplicationQueryIntent, ApplicationQueryScopeBinding,
};
use crate::application_schema::ApplicationSchema;
use crate::portable_identity::WorthQueryPortableType;

/// One model's history and its ordinary current READ entry.
/// The READ binding's scope identifies the model type; each attachment resolves
/// the actual runtime record, including its generation. This declaration grants
/// no read, mutation, attachment or navigation authority.
pub trait ApplicationHistoryBinding<Schema: ApplicationSchema>:
    WorthQueryPortableType + Sized + 'static
{
    type Access: ApplicationQueryIntent<Schema> + Clone;
    const IDENTITY: &'static str;
    const GROUP_RESOURCES: super::ApplicationHistoryGroupResources =
        super::ApplicationHistoryGroupResources::DEFAULT;
    const NAVIGATION_RESOURCES: super::ApplicationHistoryNavigationResources =
        super::ApplicationHistoryNavigationResources::DEFAULT;
}

pub type ApplicationHistoryAccessBinding<Schema, History> =
    <<History as ApplicationHistoryBinding<Schema>>::Access as ApplicationQueryIntent<Schema>>::Binding;

pub type ApplicationHistoryModel<Schema, History> = <<ApplicationHistoryAccessBinding<
    Schema,
    History,
> as ApplicationQueryBinding<Schema>>::ScopeBinding as ApplicationQueryScopeBinding<Schema>>::Scope;
