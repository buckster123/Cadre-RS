//! Parts catalog + lockfile + assembly specs.

#![deny(unsafe_code)]

mod assembly;
mod lock;
mod provider;

pub use assembly::{
    align_check, AlignExpect, AlignReport, AssemblySpec, ComponentSpec, JointSpec, PlacementSpec,
};
pub use lock::{load_parts_lock, verify_lock_entry, PartsLock, PartsLockEntry, PartsLockError};
pub use provider::{
    LocalFsProvider, PartCandidate, PartMeta, PartProvider, PartRef, ProviderError,
};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
