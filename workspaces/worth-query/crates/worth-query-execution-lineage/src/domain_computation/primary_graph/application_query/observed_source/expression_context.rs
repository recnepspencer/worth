//! Pure retained expression context value types.

use std::sync::Arc;
use worth_foundational::expression_api::{ExpressionSchema, ExpressionValue};
use worth_query_declaration::facade::application_query::ApplicationQueryExpressionSnapshot;

pub struct RetainedExpressionContext {
    snapshot: ApplicationQueryExpressionSnapshot,
}

impl RetainedExpressionContext {
    pub fn from_snapshot(snapshot: ApplicationQueryExpressionSnapshot) -> Self {
        Self { snapshot }
    }

    pub fn schema(&self) -> &ExpressionSchema {
        self.snapshot.schema()
    }

    pub fn value(&self) -> &ExpressionValue {
        self.snapshot.value()
    }

    pub fn snapshot(&self) -> &ApplicationQueryExpressionSnapshot {
        &self.snapshot
    }
}

#[derive(Clone)]
pub struct ObservedQueryExpressionContext {
    query_identifier: Arc<String>,
    context: Arc<RetainedExpressionContext>,
}

impl ObservedQueryExpressionContext {
    pub fn from_parts(
        query_identifier: Arc<String>,
        context: Arc<RetainedExpressionContext>,
    ) -> Self {
        Self {
            query_identifier,
            context,
        }
    }

    pub fn query_identifier(&self) -> &str {
        &self.query_identifier
    }

    pub fn value(&self) -> &ExpressionValue {
        self.context.value()
    }

    pub fn snapshot(&self) -> &ApplicationQueryExpressionSnapshot {
        self.context.snapshot()
    }
}
