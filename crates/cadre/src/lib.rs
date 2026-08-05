//! Cadre-RS facade crate.
//!
//! Bootstrap placeholder: workspace member so CI resolves from commit 0.
//! Logic lands in `cadre-*` crates per `docs/design.md`; this crate becomes
//! re-exports, not a monolith. Binding decisions: `docs/CHARTER.md`.

#![deny(unsafe_code)]

/// Workspace / facade version (keep in sync with package version).
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    #[test]
    fn version_is_semverish() {
        let v = super::VERSION;
        assert!(v.split('.').count() >= 2, "version={v}");
    }
}
