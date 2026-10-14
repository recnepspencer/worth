//! Attach authored delivery to one action occurrence.
use super::*;
impl ApplicationActionDeclaration {
    pub fn delivery(&self) -> Option<&ApplicationActionDeliveryDeclaration> {
        self.delivery.as_ref()
    }
    pub(crate) fn attach_fixed_delivery<Schema, Binding, Delivery>(&mut self)
    where
        Schema: crate::application_schema::ApplicationSchema,
        Binding: crate::application_operation::ApplicationMutationBinding<Schema>,
        Delivery: ApplicationFixedActionDelivery<Schema, Binding>,
    {
        self.delivery = Some(ApplicationActionDeliveryDeclaration::fixed::<
            Schema,
            Binding,
            Delivery,
        >());
    }

    pub(crate) fn attach_delivery<Schema, Binding, Delivery>(&mut self)
    where
        Schema: ApplicationSchema,
        Binding: ApplicationMutationBinding<Schema>,
        Delivery: ApplicationActionDelivery<Schema, Binding>,
    {
        self.delivery = Some(<Delivery::Mode as ApplicationActionDeliveryMode<
            Schema,
            Binding,
            Delivery,
        >>::declaration());
        if self
            .delivery
            .as_ref()
            .and_then(|d| d.fixed_output())
            .is_some()
        {
            self.mark_required_output_source();
        }
    }
}
