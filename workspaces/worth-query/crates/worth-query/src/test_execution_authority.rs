//! The sole Query unit-test execution composition root, with exclusive test custody.
use std::{
    num::NonZeroUsize,
    ops::Deref,
    sync::{Mutex, MutexGuard, OnceLock},
};
use worth_execution::{ExecutionAuthority, ExecutionAuthorityConfig};

pub(crate) struct AuthorityGuard {
    _exclusive: MutexGuard<'static, ()>,
    owner: &'static ExecutionAuthority,
}
impl Deref for AuthorityGuard {
    type Target = ExecutionAuthority;
    fn deref(&self) -> &Self::Target {
        self.owner
    }
}
pub(crate) fn authority() -> AuthorityGuard {
    static EXCLUSIVE: Mutex<()> = Mutex::new(());
    static OWNER: OnceLock<ExecutionAuthority> = OnceLock::new();
    let exclusive = EXCLUSIVE.lock().unwrap_or_else(|e| e.into_inner());
    let owner = OWNER.get_or_init(|| {
        ExecutionAuthority::try_construct(ExecutionAuthorityConfig {
            max_workers: NonZeroUsize::new(4).unwrap(),
            charged_memory_bytes: 64 * 1024 * 1024,
        })
        .expect("one Query test composition root")
    });
    AuthorityGuard {
        _exclusive: exclusive,
        owner,
    }
}
