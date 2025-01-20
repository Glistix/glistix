pub mod syntax;
use crate::analyse::TargetSupport;
use crate::ast::{SrcSpan, TypedModule};
use crate::line_numbers::LineNumbers;
use ecow::EcoString;
use camino::Utf8Path;

pub fn module(
    _module: &TypedModule,
    _line_numbers: &LineNumbers,
    _path: &Utf8Path,
    _src: &EcoString,
    _target_support: TargetSupport,
) -> Result<String, crate::Error> {
    todo!("generator not implemented")
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    Unsupported { feature: String, location: SrcSpan },
}

impl Error {
    /// Returns `true` if the error is [`Unsupported`].
    ///
    /// [`Unsupported`]: crate::nix::Error::Unsupported
    #[must_use]
    pub fn is_unsupported(&self) -> bool {
        matches!(self, Self::Unsupported { .. })
    }
}
