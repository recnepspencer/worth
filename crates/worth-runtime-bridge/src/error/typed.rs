use std::marker::PhantomData;
use std::sync::Arc;

use super::context::BridgeErrorContext;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeTypedError<K> {
    kind: K,
    message: Arc<str>,
    context: Box<BridgeErrorContext>,
}

impl<K: Copy> BridgeTypedError<K> {
    pub fn new(kind: K, message: impl Into<Arc<str>>) -> Self {
        Self {
            kind,
            message: message.into(),
            context: Box::default(),
        }
    }

    pub fn kind(&self) -> K {
        self.kind
    }

    pub(crate) fn with_context(mut self, context: BridgeErrorContext) -> Self {
        self.context = Box::new(context);
        self
    }

    pub fn context(&self) -> &BridgeErrorContext {
        &self.context
    }
}

impl<K> std::fmt::Display for BridgeTypedError<K> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl<K: std::fmt::Debug> std::error::Error for BridgeTypedError<K> {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeMessageError<Tag> {
    message: Arc<str>,
    _tag: PhantomData<Tag>,
}

impl<Tag> BridgeMessageError<Tag> {
    pub fn new(message: impl Into<Arc<str>>) -> Self {
        Self {
            message: message.into(),
            _tag: PhantomData,
        }
    }
}

impl<Tag> std::fmt::Display for BridgeMessageError<Tag> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl<Tag: std::fmt::Debug> std::error::Error for BridgeMessageError<Tag> {}
