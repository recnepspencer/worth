#![doc = include_str!("authority_compile_fail_proofs.md")]

#[cfg(test)]
mod backend_matrix;
#[cfg(test)]
mod durability_recovery;
#[cfg(test)]
mod lease_reclaim;
#[cfg(test)]
mod replication_admission;
#[cfg(test)]
mod shared_frontiers;
