// NOTE: Current dependency resolution uses simple version matching without
// a full SAT solver. For the v1.0 ecosystem (small number of packages),
// this is sufficient. A proper SAT-based resolver (like pubgrub) is
// planned for Phase 15 when the package registry launches.

pub mod manifest;
pub mod scaffold;
pub mod resolver;
pub mod lockfile;
pub mod commands;
