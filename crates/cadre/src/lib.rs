//! Cadre-RS facade crate.
//!
//! Re-exports workspace libraries. Logic lives in `cadre-*` crates
//! (`docs/design.md`). Binding decisions: `docs/CHARTER.md`.

#![deny(unsafe_code)]

pub use cadre_kernel as kernel;

/// Workspace facade version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use cadre_kernel::{GeomKernel, MockKernel, Placement};

    #[test]
    fn version_is_semverish() {
        let v = super::VERSION;
        assert!(v.split('.').count() >= 2, "version={v}");
    }

    #[test]
    fn facade_reexports_kernel() {
        let mut k = MockKernel::new();
        let id = k
            .box_solid(1.0, 2.0, 3.0, Placement::IDENTITY)
            .expect("box");
        let f = k.facts(id).expect("facts");
        assert!((f.volume_mm3 - 6.0).abs() < 1e-12);
    }
}
