/// Register mapping subsystem split into focused components:
/// - `core`: mapping types and invariants
/// - `builder`: hand-authored construction API
/// - `validate`: overlap and invariant checks
mod builder;
mod core;
mod validate;

pub(crate) use core::{MapTarget, MappingPlan, RegisterMapping, XmmLane};
