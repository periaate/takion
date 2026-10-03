use crate::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pipe<Seeker, Parser>(pub Seeker, pub Parser);

impl<S: Rule, P: Rule> Rule for Pipe<S, P> {
    type This = P::This;
}

impl<'a, T: 'a, S: Parse<'a, T>, P: Parse<'a, T>> Parse<'a, T> for Pipe<S, P> {
    const NULLABLE: bool = S::NULLABLE;
    type Item = P::Item;

    #[inline]
    fn parse<Cx: Ctx>(cursor: Cursor<'a, T>) -> Ret<'a, T, Self::Item, Cx> {
        // Evaluate Seeker internally as a bounded slice
        match <Span<S>>::parse::<Cx>(cursor) {
            Ret::Pass(new_cursor, span) => {
                let inner_cursor = Cursor { index: 0, source: span };
                match P::parse::<Cx>(inner_cursor) {
                    Ret::Pass(_, val) => Ret::Pass(new_cursor, val),
                    Ret::Miss(info) => Ret::Miss(info.wrap::<Self>(cursor.index, new_cursor.index)),
                    Ret::Fail(info) => Ret::Fail(info.wrap::<Self>(cursor.index, new_cursor.index)),
                }
            }
            Ret::Miss(info) => Ret::Miss(info),
            Ret::Fail(info) => Ret::Fail(info),
        }
    }
}


#[test]
fn pipe() {
    
}

