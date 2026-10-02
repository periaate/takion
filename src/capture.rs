use super::*;

pub struct Span<P>(PhantomData<P>);

impl<P: Rule> Rule for Span<P> { type This = P::This; }

impl<'a, T: 'a, P: Parse<'a, T>> Parse<'a, T> for Span<P> {
    const NULLABLE: bool = P::NULLABLE;
    type Item = &'a [T];

    #[inline(always)]
    fn parse<Cx: Ctx>(cursor: Cursor<'a, T>) -> Ret<'a, T, Self::Item, Cx> {
        let current = P::parse::<Cx>(cursor)??.0;
        Ret::Pass(current, cursor.slice_between(&current))
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Skip<P>(pub P);

impl<P: Rule> Rule for Skip<P> { type This = P::This; }

impl<'a, T, P: Parse<'a, T>> Parse<'a, T> for Skip<P> {
    const NULLABLE: bool = P::NULLABLE;
    type Item = ();

    #[inline(always)]
    fn parse<Cx: Ctx>(cursor: Cursor<'a, T>) -> Ret<'a, T, Self::Item, Cx> {
        Pass(P::parse::<Cx>(cursor)??.0, ())
    }
}
