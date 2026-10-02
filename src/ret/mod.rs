use super::*;

mod try_impl;
mod util_impl;

/// Ret implements `?` for `Ret`, resolves to `SubRet`
pub enum Ret<'a, Unit, Match, Cx: Ctx = Normal> {
    Pass(Cursor<'a, Unit>, Match),
    Miss(Cx::Info<'a, Unit>),
    Fail(Cx::Info<'a, Unit>),
}

/// SubRet implements `?` for `Ret`, resolves to `(Cursor, Match)`
pub enum SubRet<'a, Unit, Match, Cx: Ctx = Normal> {
    Pass(Cursor<'a, Unit>, Match),
    Miss(Cx::Info<'a, Unit>),
}

