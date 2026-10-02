use crate::macros::impl_tuple::impl_tuples;
use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
/// Seq accepts tuples and is identical operationally to a free tuple.
pub struct Seq<T>(pub T);

impl_tuples!(sequence);

macro sequence($($id:ident)*) {
    #[expect(non_snake_case, reason = "idents used are type params.")]
    impl<'a, T, $($id,)*> Parse<'a, T> for Seq<($($id,)*)>
    where $($id: Parse<'a, T> + Rule,)* {
        const NULLABLE: bool = $($id::NULLABLE)&&*;
        type Item = ($($id::Item,)*);

        #[inline]
        fn parse<Cx: Ctx>(start: Cursor<'a, T>) -> Ret<'a, T, Self::Item, Cx> {
            let cursor = start;
            $(let (cursor, $id) = $id::parse::<Cx>(cursor)
                    .map_err(|err| err.wrap::<Self>(start.index, cursor.index))??;)*
            Pass(cursor, ($($id,)*))
        }
    }

    #[expect(non_snake_case, reason = "idents used are type params.")]
    impl<'a, T, $($id,)*> Parse<'a, T> for ($($id,)*)
    where $($id: Parse<'a, T> + Rule,)* {
        const NULLABLE: bool = $($id::NULLABLE)&&*;
        type Item = ($($id::Item,)*);

        #[inline]
        fn parse<Cx: Ctx>(start: Cursor<'a, T>) -> Ret<'a, T, Self::Item, Cx> {
            let cursor = start;
            $(let (cursor, $id) = $id::parse::<Cx>(cursor)
                    .map_err(|err| err.wrap::<Self>(start.index, cursor.index))??;)*
            Pass(cursor, ($($id,)*))
        }
    }


    impl<$($id: Rule,)*> FormatType for ($($id,)*) {
        fn fmt_type(f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", [$([
                { let r = <<$id as Rule>::This>::realize();
                  if r == "\"\"\"" { "'\"'".to_string() } else { r } },
                <<$id as Rule>::Mod>::realize(),
            ].join(""),)*].join(" ").trim())
        }
    }

    impl<$($id: Rule,)*> Rule for ($($id,)*) { type This = Self; }
    impl<$($id: Rule,)*> Rule for Seq<($($id,)*)> { type This = ($($id,)*); }
}


#[derive(Clone, Copy, PartialEq, Eq)]
 pub struct Join<Par, Sep = Txt<"">>(pub Par, pub Sep);

impl_tuples!(impl_join);

macro impl_join($($id:ident)*) {
    impl<J: Rule + Literal, $($id: Rule,)*> FormatType for Join<($($id,)*), J> {
        fn fmt_type(f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", [
                $( <$id as Rule>::Fmt::realize(), )*
            ].join(J::literal()).trim())
        }
    }

    impl<J: Rule + Literal, $($id: Rule,)*> Rule for Join<($($id,)*), J> {
        type This = Self;
        type Fmt = Self;
    }
}

