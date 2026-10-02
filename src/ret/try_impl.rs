use std::ops::{ControlFlow, FromResidual, Residual, Try};
use super::*;

#[test]
fn try_ret_subret() {
    use std::assert_matches;

    fn cur(s: &[u8]) -> Cursor<'_, u8> { Cursor { index: 0, source: s } }
    fn info(s: &[u8]) -> Frame<'_, u8> { Frame::new::<Tok<"some">>(cur(s), cur(s).index) }
    let s = b"x";

    // --- Ret::branch ---
    assert_matches!(Ret::<u8, u32, Traced>::Pass(cur(s), 1).branch(), ControlFlow::Continue(SubRet::Pass(..)));
    assert_matches!(Ret::<u8, u32, Traced>::Miss(info(s)).branch(),   ControlFlow::Continue(SubRet::Miss(_)));
    assert_matches!(Ret::<u8, u32, Traced>::Fail(info(s)).branch(),   ControlFlow::Break(RetFail(_)));

    assert_matches!(SubRet::<u8, u32, Traced>::Pass(cur(s), 1).branch(), ControlFlow::Continue(((_, 1))));
    assert_matches!(SubRet::<u8, u32, Traced>::Miss(info(s)).branch(),   ControlFlow::Break(SubRetMiss(..)));

    fn ret_qmark(v: Ret<u8, u32, Traced>) -> Ret<u8, u32, Traced> {
        let sub: SubRet<u8, u32, Traced> = v?;
        match sub {
            SubRet::Pass(c, m) => Ret::Pass(c, m + 1),
            SubRet::Miss(_)    => Ret::Pass(cur(b"x"), 0),
        }
    }
    assert_matches!(ret_qmark(Ret::Pass(cur(s), 9)), Ret::Pass(_, 10));
    assert_matches!(ret_qmark(Ret::Miss(info(s))),   Ret::Pass(_, 0));
    assert_matches!(ret_qmark(Ret::Fail(info(s))),   Ret::Fail(_));

    fn subret_in_ret(v: SubRet<u8, u32, Traced>) -> Ret<u8, u32, Traced> {
        let (c, m): (Cursor<u8>, u32) = v?;
        Ret::Pass(c, m * 2)
    }
    assert_matches!(subret_in_ret(SubRet::Pass(cur(s), 5)), Ret::Pass(_, 10));
    assert_matches!(subret_in_ret(SubRet::Miss(info(s))),   Ret::Miss(_));

    fn subret_in_subret(v: SubRet<u8, u32, Traced>) -> SubRet<u8, u32, Traced> {
        let (c, m): (Cursor<u8>, u32) = v?;
        SubRet::Pass(c, m + 1)
    }
    assert_matches!(subret_in_subret(SubRet::Pass(cur(s), 4)), SubRet::Pass(_, 5));
    assert_matches!(subret_in_subret(SubRet::Miss(info(s))),   SubRet::Miss(_));
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RetFail<Info>(pub Info);

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SubRetMiss<Info, C: Ctx>(pub Info, pub(crate) PhantomData<C>);

impl<I: Debug, C: Ctx> Debug for SubRetMiss<I, C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("SubRetMiss").field(&self.0).finish()
    }
}

impl<'a, Unit, Match, Cx: Ctx> Try for Ret<'a, Unit, Match, Cx> {
    type Output = SubRet<'a, Unit, Match, Cx>;
    type Residual = RetFail<Cx::Info<'a, Unit>>;

    #[inline(always)]
    fn from_output(output: Self::Output) -> Self {
        match output {
            SubRet::Pass(c, m) => Ret::Pass(c, m),
            SubRet::Miss(info) => Ret::Miss(info),
        }
    }

    #[inline(always)]
    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        match self {
            Ret::Pass(c, m) => ControlFlow::Continue(SubRet::Pass(c, m)),
            Ret::Miss(info) => ControlFlow::Continue(SubRet::Miss(info)),
            Ret::Fail(info) => ControlFlow::Break(RetFail(info)),
        }
    }
}

impl<'a, Unit, Match, Cx: Ctx> Residual<SubRet<'a, Unit, Match, Cx>> for RetFail<Cx::Info<'a, Unit>> {
    type TryType = Ret<'a, Unit, Match, Cx>;
}

impl<'a, Unit, Match, Cx: Ctx> FromResidual<RetFail<Cx::Info<'a, Unit>>> for Ret<'a, Unit, Match, Cx> {
    #[inline(always)]
    fn from_residual(residual: RetFail<Cx::Info<'a, Unit>>) -> Self {
        Ret::Fail(residual.0)
    }
}

impl<'a, Unit, Match, Cx: Ctx> Try for SubRet<'a, Unit, Match, Cx> {
    type Output = (Cursor<'a, Unit>, Match);
    type Residual = SubRetMiss<Cx::Info<'a, Unit>, Cx>;

    #[inline(always)]
    fn from_output(output: Self::Output) -> Self {
        SubRet::Pass(output.0, output.1)
    }

    #[inline(always)]
    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        match self {
            SubRet::Pass(c, m) => ControlFlow::Continue((c, m)),
            SubRet::Miss(info) => ControlFlow::Break(SubRetMiss(info, PhantomData)),
        }
    }
}

impl<'a, Unit, Match, C: Ctx> Residual<(Cursor<'a, Unit>, Match)> for SubRetMiss<C::Info<'a, Unit>, C>
{
    type TryType = SubRet<'a, Unit, Match, C>;
}

impl<'a, Unit, Match, Cx: Ctx> FromResidual<SubRetMiss<Cx::Info<'a, Unit>, Cx>> for Ret<'a, Unit, Match, Cx> {
    #[inline(always)]
    fn from_residual(residual: SubRetMiss<Cx::Info<'a, Unit>, Cx>) -> Self {
        Ret::Miss(residual.0)
    }
}

impl<'a, Unit, Match, Cx: Ctx> FromResidual<SubRetMiss<Cx::Info<'a, Unit>, Cx>> for SubRet<'a, Unit, Match, Cx> {
    #[inline(always)]
    fn from_residual(residual: SubRetMiss<Cx::Info<'a, Unit>, Cx>) -> Self {
        SubRet::Miss(residual.0)
    }
}
