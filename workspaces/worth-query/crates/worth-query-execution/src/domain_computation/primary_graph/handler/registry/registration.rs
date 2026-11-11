//! Installed handler origin distinguishes compiled meaning from supplied implementation.
use super::*;

#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) enum HandlerOrigin {
    Supplier,
    CompiledBody,
}

impl<Schema: ApplicationSchema> PendingMutationHandlerRegistry<Schema> {
    pub(super) fn register_resolved<Binding, Handler>(
        &mut self,
        binding_identity: &ApplicationSchemaBindingIdentity,
        installed: &WorthQueryInstalledApplicationMutationBinding<Schema, Binding>,
        handler: Handler,
        origin: HandlerOrigin,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial>
    where
        Binding: ApplicationMutationBinding<Schema>,
        Handler: OperationHandler<Schema, Binding>,
    {
        if installed.operation().binding_identity() != binding_identity {
            return Err(denial(
                DenialKind::ForeignMutationHandler,
                Binding::IDENTITY,
            ));
        }
        if installed.identity() != Binding::IDENTITY
            || installed.handler_identity() != Binding::HANDLER_IDENTITY
        {
            return Err(denial(
                DenialKind::MutationHandlerMeaningMismatch,
                Binding::IDENTITY,
            ));
        }
        if Binding::WORKFLOW_CONTROL && origin != HandlerOrigin::CompiledBody {
            return Err(denial(
                DenialKind::WorkflowControlHandler,
                Binding::IDENTITY,
            ));
        }
        if self.entries.contains_key(Binding::IDENTITY) {
            return Err(denial(
                DenialKind::DuplicateMutationHandler,
                Binding::IDENTITY,
            ));
        }
        self.entries.insert(
            Binding::IDENTITY.to_owned(),
            PendingMutationHandler::retain_original::<Schema, Binding, Handler>(handler, origin),
        );
        Ok(())
    }
}

impl PendingMutationHandler {
    pub(super) fn retain_original<Schema, Binding, Handler>(
        handler: Handler,
        origin: HandlerOrigin,
    ) -> Self
    where
        Schema: ApplicationSchema,
        Binding: ApplicationMutationBinding<Schema>,
        Handler: OperationHandler<Schema, Binding>,
    {
        Self {
            identity: Binding::IDENTITY,
            handler_identity: Binding::HANDLER_IDENTITY.to_owned(),
            decision_type: TypeId::of::<Binding::Decision>(),
            denial_type: TypeId::of::<Binding::Denial>(),
            value: Arc::new(TypedHandlerSlot::<Schema, Binding> {
                handler: Arc::new(handler),
            }),
            preflight_home_retirement: super::retirement::preflight_original_handler::<
                Schema,
                Binding,
            >,
            advance_home_retirement: super::retirement::advance_original_handler::<Schema, Binding>,
            durable_capabilities: if origin == HandlerOrigin::CompiledBody {
                Some(&[])
            } else {
                Handler::declared_durable_capabilities()
            },
            origin,
        }
    }
}
