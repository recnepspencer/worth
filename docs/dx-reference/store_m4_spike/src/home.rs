use crate::{
    AccessScope, AppError, CommitId, DiagnosticReport, HistoricalReadDenial, Principal,
    ProductRevision, Program, ReclaimBudget, ReclamationReport, Reportable, ResidencyBudget,
    ResidencyPressureSnapshot, RetainedBasisToken,
};
use std::marker::PhantomData;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductName(&'static str);

impl ProductName {
    pub const fn main() -> Self {
        Self("main")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BranchName(String);

impl BranchName {
    pub fn new(name: &str) -> Result<Self, AppError> {
        if name.is_empty() {
            return Err(AppError::InvalidName);
        }
        Ok(Self(name.to_owned()))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IdempotencyKey(u64);

#[allow(non_upper_case_globals)]
pub const key_a: IdempotencyKey = IdempotencyKey(1);
#[allow(non_upper_case_globals)]
pub const key_b: IdempotencyKey = IdempotencyKey(2);
#[allow(non_upper_case_globals)]
pub const key_advance: IdempotencyKey = IdempotencyKey(3);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BranchEdit;

pub const fn edit_option_a() -> BranchEdit {
    BranchEdit
}

pub const fn edit_option_b() -> BranchEdit {
    BranchEdit
}

pub const fn advance_main_many_times() -> BranchEdit {
    BranchEdit
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProductHead {
    pub revision: ProductRevision,
}

pub struct HomeBuilder<P> {
    _app: PhantomData<P>,
    budget: Option<ResidencyBudget>,
}

impl<P> HomeBuilder<P> {
    pub fn residency_budget(mut self, budget: ResidencyBudget) -> Self {
        self.budget = Some(budget);
        self
    }

    pub fn open(self) -> Result<Home<P>, AppError> {
        Ok(Home {
            _app: PhantomData,
            _budget: self.budget,
        })
    }
}

pub struct Home<P> {
    _app: PhantomData<P>,
    _budget: Option<ResidencyBudget>,
}

impl<P: Program> Home<P> {
    pub fn directory(_home_dir: &Path) -> HomeBuilder<P> {
        HomeBuilder {
            _app: PhantomData,
            budget: None,
        }
    }

    pub fn request<'req>(
        &self,
        _principal: &'req Principal,
        _scope: &'req AccessScope,
    ) -> Result<Request<'req, P::Schema>, AppError> {
        Ok(Request {
            _home: PhantomData,
            _schema: PhantomData,
        })
    }

    pub fn product<'a>(
        &'a self,
        _name: &ProductName,
    ) -> Result<ProductRef<'a, P::Schema>, AppError> {
        Ok(ProductRef {
            _home: PhantomData,
            _schema: PhantomData,
        })
    }

    pub fn checkpoint(&self, _req: &Request<'_, P::Schema>) -> Result<(), AppError> {
        Ok(())
    }

    pub fn residency_report(
        &self,
        _req: &Request<'_, P::Schema>,
    ) -> Result<ResidencyPressureSnapshot, AppError> {
        Ok(ResidencyPressureSnapshot::default())
    }

    pub fn reclaim_step(
        &self,
        _req: &Request<'_, P::Schema>,
        _budget: ReclaimBudget,
    ) -> Result<ReclamationReport, AppError> {
        Ok(ReclamationReport::default())
    }

    pub fn close(self, _req: &Request<'_, P::Schema>) -> Result<(), AppError> {
        Ok(())
    }
}

pub struct Request<'a, S> {
    _home: PhantomData<&'a ()>,
    _schema: PhantomData<S>,
}

impl<S> Request<'_, S> {
    pub fn explain(&self, err: &dyn Reportable) -> DiagnosticReport {
        DiagnosticReport {
            disposition: err.disposition(),
        }
    }
}

pub struct ProductRef<'a, S> {
    _home: PhantomData<&'a ()>,
    _schema: PhantomData<S>,
}

impl<'a, S> ProductRef<'a, S> {
    pub fn head_view(&self) -> HeadOnlyView<'a, S> {
        HeadOnlyView {
            _home: PhantomData,
            _schema: PhantomData,
        }
    }

    pub fn scan<R>(
        &self,
        _req: &Request<'_, S>,
        _filter: ScanFilter,
    ) -> Result<RecordScan<'_, R>, AppError> {
        Ok(RecordScan {
            _owner: PhantomData,
            _record: PhantomData,
        })
    }

    pub fn observe(&self, _req: &Request<'_, S>) -> Result<ProductHead, AppError> {
        Ok(ProductHead {
            revision: ProductRevision(1),
        })
    }

    pub fn create_branch(
        &self,
        _req: &Request<'_, S>,
        _name: BranchName,
        _at: ProductRevision,
    ) -> Result<BranchRef<'a, S>, AppError> {
        Ok(BranchRef {
            _home: PhantomData,
            _schema: PhantomData,
        })
    }

    pub fn mutate(
        &self,
        _req: &Request<'_, S>,
        _edit: BranchEdit,
        _key: IdempotencyKey,
    ) -> Result<(), AppError> {
        Ok(())
    }

    pub fn archive_branch(
        &self,
        _req: &Request<'_, S>,
        _name: &BranchName,
    ) -> Result<(), AppError> {
        Ok(())
    }

    pub fn delete_branch(&self, _req: &Request<'_, S>, _name: &BranchName) -> Result<(), AppError> {
        Ok(())
    }

    pub fn at_revision(
        &self,
        _req: &Request<'_, S>,
        _revision: ProductRevision,
    ) -> Result<HistoricalView<'a, S>, AppError> {
        Ok(HistoricalView {
            _home: PhantomData,
            _schema: PhantomData,
        })
    }

    pub fn resume_cursor<T>(
        &self,
        _req: &Request<'_, S>,
        token: &RetainedBasisToken,
    ) -> Result<HistoricalCursor<'a, S, T>, HistoricalReadDenial> {
        Ok(HistoricalCursor {
            _loan: ReaderLoan {
                _owner: PhantomData,
            },
            _schema: PhantomData,
            _item: PhantomData,
            token: token.clone(),
        })
    }
}

pub struct HeadOnlyView<'a, S> {
    _home: PhantomData<&'a ()>,
    _schema: PhantomData<S>,
}

impl<S> HeadOnlyView<'_, S> {
    pub fn observe_head(&self, _req: &Request<'_, S>) -> ProductHead {
        ProductHead {
            revision: ProductRevision(1),
        }
    }
}

pub struct BranchRef<'a, S> {
    _home: PhantomData<&'a ()>,
    _schema: PhantomData<S>,
}

impl<S> BranchRef<'_, S> {
    pub fn mutate(
        &self,
        _req: &Request<'_, S>,
        _edit: BranchEdit,
        _key: IdempotencyKey,
    ) -> Result<(), AppError> {
        Ok(())
    }
}

pub struct LargeMesh;

pub struct LargeMeshColumns {
    pub geometry_bytes: FieldHandle<LargeMesh>,
}

impl LargeMesh {
    pub const COL: LargeMeshColumns = LargeMeshColumns {
        geometry_bytes: FieldHandle(PhantomData),
    };
}

pub struct FieldHandle<R>(PhantomData<R>);

pub struct ScanFilter;

impl ScanFilter {
    pub const fn by_kind() -> Self {
        Self
    }
}

pub struct RecordScan<'a, R> {
    _owner: PhantomData<&'a ()>,
    _record: PhantomData<R>,
}

impl<'a, R> RecordScan<'a, R> {
    pub fn next_row<S>(&mut self, _req: &Request<'_, S>) -> Result<Option<RowView<'_>>, AppError> {
        Ok(None)
    }
}

pub struct RowView<'loan> {
    _loan: PhantomData<&'loan ()>,
}

impl<'loan> RowView<'loan> {
    pub fn stream_field<S, R>(
        &self,
        _req: &Request<'_, S>,
        _field: FieldHandle<R>,
    ) -> Result<ValueReader<'loan>, AppError> {
        Ok(ValueReader { _loan: PhantomData })
    }
}

/// Bounded streaming reader — intentionally exposes ONLY fixed-buffer `read_chunk`,
/// never unbounded whole-value materialization.
pub struct ValueReader<'loan> {
    _loan: PhantomData<&'loan ()>,
}

impl ValueReader<'_> {
    pub fn read_chunk(&mut self, _buf: &mut [u8]) -> Result<usize, AppError> {
        Ok(0)
    }
}

pub struct ReaderLoan<'a> {
    _owner: PhantomData<&'a ()>,
}

pub struct HistoricalView<'a, S> {
    _home: PhantomData<&'a ()>,
    _schema: PhantomData<S>,
}

impl<'a, S> HistoricalView<'a, S> {
    pub fn scan_history<T>(
        &self,
        _req: &Request<'_, S>,
        _query: AuditQuery,
    ) -> Result<HistoricalCursor<'a, S, T>, AppError> {
        Ok(HistoricalCursor {
            _loan: ReaderLoan {
                _owner: PhantomData,
            },
            _schema: PhantomData,
            _item: PhantomData,
            token: RetainedBasisToken {
                commit_id: CommitId(1),
                root_semantic_commitment: [0u8; 32],
                query_digest: [0u8; 32],
            },
        })
    }
}

pub struct AuditEntry;

pub struct AuditQuery;

impl AuditQuery {
    pub const fn ordered() -> Self {
        Self
    }
}

pub struct BorrowedHistoricalRow<'loan, T> {
    _loan: PhantomData<&'loan T>,
}

pub struct HistoricalCursor<'a, S, T> {
    _loan: ReaderLoan<'a>,
    _schema: PhantomData<S>,
    _item: PhantomData<T>,
    token: RetainedBasisToken,
}

impl<'a, S, T> HistoricalCursor<'a, S, T> {
    pub fn next_page(&mut self, _req: &Request<'_, S>, _limit: usize) -> Result<Vec<T>, AppError> {
        Ok(Vec::new())
    }

    pub fn peek_borrowed_row(&self) -> Option<BorrowedHistoricalRow<'_, T>> {
        None
    }

    pub fn checkpoint_token(&self) -> RetainedBasisToken {
        self.token.clone()
    }
}
