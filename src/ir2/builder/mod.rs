// Block creation + switch_to
mod blocks;
// Core builder state + emit/terminator plumbing
mod core;
// Effect op helpers + type checks
mod effect;
// Exit tracking + require_single_exit
mod exit;
// Pure op helpers + type checks
mod pure;
// Builder unit tests
mod tests;
// Shared validation helpers
mod validate;

pub use core::IrBuilder;
