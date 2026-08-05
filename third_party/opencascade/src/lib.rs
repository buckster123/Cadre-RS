use thiserror::Error;

pub mod adhoc;
pub mod angle;
pub mod mesh;
pub mod primitives;
pub mod workplane;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed to write STL file")]
    StlWriteFailed,
    #[error("Failed to read STEP file")]
    StepReadFailed,
    #[error("Failed to write STEP file")]
    StepWriteFailed,
    #[error("BRep transform failed")]
    TransformFailed,
    #[error("Primitive construction failed")]
    PrimitiveFailed,
    #[error("Fillet failed (radius too large or edges unsuitable)")]
    FilletFailed,
    #[error("Chamfer failed (distance too large or edges unsuitable)")]
    ChamferFailed,
}
