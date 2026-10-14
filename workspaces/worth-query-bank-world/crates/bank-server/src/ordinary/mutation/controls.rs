use bank_domain::proposals::BankIdempotencyKey;
use worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope;

#[derive(Clone)]
pub struct BankMutationControls {
    request: WorthQueryRequestScope,
    idempotency_key: BankIdempotencyKey,
}

impl BankMutationControls {
    pub const fn new(request: WorthQueryRequestScope, idempotency_key: BankIdempotencyKey) -> Self {
        Self {
            request,
            idempotency_key,
        }
    }

    pub const fn request(&self) -> &WorthQueryRequestScope {
        &self.request
    }

    pub const fn idempotency_key(&self) -> &BankIdempotencyKey {
        &self.idempotency_key
    }

    pub(in crate::ordinary::mutation) fn into_parts(
        self,
    ) -> (WorthQueryRequestScope, BankIdempotencyKey) {
        (self.request, self.idempotency_key)
    }
}
