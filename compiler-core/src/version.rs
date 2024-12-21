/// The current version of the gleam compiler. If this does not match what is
/// already in the build folder we will not reuse any cached artifacts and
/// instead build from scratch
/// Note that this should be updated to correspond to the Gleam version
/// we are basing Glistix on. This is checked by packages.
pub const COMPILER_VERSION: &str = "1.5.1";
