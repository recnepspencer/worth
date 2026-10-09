//! The sole Query unit-test execution composition root, with exclusive test custody.
use std::{
    num::NonZeroUsize,
    ops::Deref,
    sync::{Arc, Mutex, MutexGuard, OnceLock},
};
use worth_execution::{ExecutionAuthority, ExecutionAuthorityConfig};

pub(crate) struct AuthorityGuard {
    _exclusive: MutexGuard<'static, ()>,
    owner: &'static Arc<ExecutionAuthority>,
}
impl AuthorityGuard {
    pub(crate) fn shared_owner(&self) -> Arc<ExecutionAuthority> {
        Arc::clone(self.owner)
    }
}
impl Deref for AuthorityGuard {
    type Target = ExecutionAuthority;
    fn deref(&self) -> &Self::Target {
        self.owner
    }
}
pub(crate) fn authority() -> AuthorityGuard {
    static EXCLUSIVE: Mutex<()> = Mutex::new(());
    static OWNER: OnceLock<Arc<ExecutionAuthority>> = OnceLock::new();
    let exclusive = EXCLUSIVE.lock().unwrap_or_else(|e| e.into_inner());
    let owner = OWNER.get_or_init(|| {
        Arc::new(
            ExecutionAuthority::try_construct(ExecutionAuthorityConfig {
                max_workers: NonZeroUsize::new(4).unwrap(),
                charged_memory_bytes: Some(64 * 1024 * 1024),
            })
            .expect("one Query test composition root"),
        )
    });
    AuthorityGuard {
        _exclusive: exclusive,
        owner,
    }
}
