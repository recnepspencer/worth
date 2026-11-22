//! Pure batch read denial enum.

use super::super::WorthQueryApplicationQueryBatchResourceDenial;
use super::WorthQueryApplicationOneShotDenial;

#[derive(Debug)]
pub enum WorthQueryApplicationBatchReadDenial {
    Resource(WorthQueryApplicationQueryBatchResourceDenial),
    Execution(WorthQueryApplicationOneShotDenial),
}
