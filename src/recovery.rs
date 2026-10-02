use super::*;

pub type Commit<First, Then> = destruct!( (_, body) = (First, Cut<Then>) );

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cut<P>(pub P);

impl<R: Rule> Rule for Cut<R> {
    type This = R::This;
    type Mod = R::Mod;
    type Opt = R::Opt;
}

impl<'a, T, P: Parse<'a, T>> Parse<'a, T> for Cut<P> {
    const NULLABLE: bool = P::NULLABLE;
    type Item = P::Item;
    #[inline]
    fn parse<Cx: Ctx>(cursor: Cursor<'a, T>) -> Ret<'a, T, Self::Item, Cx> {
        match P::parse::<Cx>(cursor) {
            Pass(c, v) => Pass(c, v),
            Fail(err) | Miss(err) => Fail(err),
        }
    }
}
impl<'a, T, K: Parse<'a, T>> Parse<'a, T> for Option<K> {
    const NULLABLE: bool = true;
    type Item = Option<K::Item>;

    #[inline]
    fn parse<Cx: Ctx>(cursor: Cursor<'a, T>) -> Ret<'a, T, Self::Item, Cx> {
        match K::parse::<Cx>(cursor)? {
            SubRet::Pass(c, value) => Pass(c, Some(value)),
            SubRet::Miss(_) => Pass(cursor, None),
        }
    }
}

impl<T: Rule> Rule for Option<T> {
    type This = T::This;
    type Mod = T::Opt;
    type Opt = T::Opt;
    type Fmt = Join<(Self::This, Self::Mod)>;
}
