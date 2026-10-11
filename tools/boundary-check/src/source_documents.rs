//! One immutable source acquisition per file for a constitutional scan.
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::io;
use std::path::{Path, PathBuf};
use std::rc::Rc;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Counts {
    pub(crate) reads: usize,
    pub(crate) parses: usize,
}
#[derive(Default)]
struct Documents {
    aliases: BTreeMap<PathBuf, PathBuf>,
    bytes: BTreeMap<PathBuf, Result<Rc<[u8]>, ReadFailure>>,
    syntax: BTreeMap<PathBuf, syn::Result<Rc<syn::File>>>,
    counts: Counts,
}
#[derive(Clone)]
struct ReadFailure {
    kind: io::ErrorKind,
    os_code: Option<i32>,
    message: String,
}
impl ReadFailure {
    fn capture(error: io::Error) -> Self {
        Self {
            kind: error.kind(),
            os_code: error.raw_os_error(),
            message: error.to_string(),
        }
    }
    fn restore(&self) -> io::Error {
        match self.os_code {
            Some(code) => io::Error::from_raw_os_error(code),
            None => io::Error::new(self.kind, self.message.clone()),
        }
    }
}
impl Documents {
    fn key(&mut self, path: &Path) -> PathBuf {
        self.aliases
            .entry(path.to_path_buf())
            .or_insert_with(|| std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf()))
            .clone()
    }
}
thread_local! {
    static CURRENT: RefCell<Option<Rc<RefCell<Documents>>>> = const { RefCell::new(None) };
}
/// Rules share inputs within this lifetime; another scan observes fresh inputs.
pub(crate) struct Session {
    documents: Rc<RefCell<Documents>>,
    previous: Option<Rc<RefCell<Documents>>>,
}
impl Session {
    pub(crate) fn new() -> Self {
        let documents = Rc::new(RefCell::new(Documents::default()));
        let previous = CURRENT.with(|slot| slot.replace(Some(Rc::clone(&documents))));
        Self {
            documents,
            previous,
        }
    }
    pub(crate) fn counts(&self) -> Counts {
        self.documents.borrow().counts
    }
}
impl Drop for Session {
    fn drop(&mut self) {
        CURRENT.with(|slot| slot.replace(self.previous.take()));
    }
}
fn current() -> Option<Rc<RefCell<Documents>>> {
    CURRENT.with(|slot| slot.borrow().as_ref().map(Rc::clone))
}
fn bytes(path: &Path) -> io::Result<Rc<[u8]>> {
    let Some(documents) = current() else {
        return std::fs::read(path).map(Rc::from);
    };
    let key = documents.borrow_mut().key(path);
    if let Some(cached) = documents.borrow().bytes.get(&key) {
        return cached.clone().map_err(|error| error.restore());
    }
    if path.extension().is_some_and(|value| value == "rs") {
        documents.borrow_mut().counts.reads += 1;
    }
    let result = crate::scan_profile::measure("physical file read", || std::fs::read(path))
        .map(Rc::<[u8]>::from)
        .map_err(ReadFailure::capture);
    documents.borrow_mut().bytes.insert(key, result.clone());
    result.map_err(|error| error.restore())
}
pub(crate) fn read(path: impl AsRef<Path>) -> io::Result<Vec<u8>> {
    bytes(path.as_ref()).map(|bytes| bytes.to_vec())
}
pub(crate) fn read_to_string(path: impl AsRef<Path>) -> io::Result<String> {
    let bytes = bytes(path.as_ref())?;
    std::str::from_utf8(&bytes).map(str::to_owned).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "stream did not contain valid UTF-8",
        )
    })
}
pub(crate) fn record_parse() {
    if let Some(documents) = current() {
        documents.borrow_mut().counts.parses += 1;
    }
}
pub(crate) fn parsed(
    path: &Path,
    operation: impl FnOnce() -> syn::Result<syn::File>,
) -> syn::Result<syn::File> {
    let Some(documents) = current() else {
        return operation();
    };
    let key = documents.borrow_mut().key(path);
    if let Some(cached) = documents.borrow().syntax.get(&key) {
        return cached
            .as_ref()
            .map(|syntax| syntax.as_ref().clone())
            .map_err(Clone::clone);
    }
    // Parsing may update the work count; release the cache borrow first.
    let result = operation().map(Rc::new);
    documents.borrow_mut().syntax.insert(key, result.clone());
    result.map(|syntax| syntax.as_ref().clone())
}
#[cfg(test)]
mod tests;
