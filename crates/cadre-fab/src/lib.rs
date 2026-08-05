//! Fabrication path: DXF, DFM, slicer orch, gcode-check, printers.

#![deny(unsafe_code)]

mod dfm;
mod dxf;
mod face_dxf;
mod gcode;
mod printer;
mod slicer;

pub use dfm::{
    check_dfm, load_profile_json, sendcutsend_laser_v1, DfmFinding, DfmProfile, DfmReport,
    DfmSeverity, FlatPart,
};
pub use dxf::{plate_with_holes_dxf, write_dxf_r12, DxfEntity, DxfLayer};
pub use face_dxf::{face_to_dxf, FaceDxfReport, FacePick};
pub use gcode::{check_gcode, GcodeFlavor, GcodeReport, PrinterVolume};
pub use printer::{
    evaluate_start_gates, hex_sha256, BambuAdapter, BambuTransport, DryRunReport,
    ExternalLiveTransport, NullTransport, Printer, PrinterError, PrinterInfo, RecordingTransport,
    StartGate, StartRequest, CONFIRM_START,
};
pub use slicer::{discover_slicers, slice_command_preview, SlicerInfo, SlicerKind};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
