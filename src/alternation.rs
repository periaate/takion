use super::*;

/// Tries each alternative in order, returning the first match as an `AltN` enum.
/// ```
/// use takion::*;
///
/// type YesNo = Alt<(Tok<"yes">, Tok<"no">)>;
/// assert!(YesNo::parse::<()>("no".into()).is_pass());
/// assert!(YesNo::parse::<()>("maybe".into()).is_miss());
/// ```
///
/// ```
/// use takion::alternation::{ Alt, Alternation, Alt3 };
/// 
/// type Parser = Alt<((), bool, &'static str)>;
///
/// let this: <Parser as Alternation>::Alternate = Alt3::A(());
/// let this: Alt3<(), bool, &'static str> = this;
/// ```
pub struct Alt<T>(pub T);

/// Maps `Alt<(A, B, ..)>` to its item enum, `AltN<A, B, ..>`.
pub trait Alternation { type Alternate; }

/// Collapses an `AltN` whose alternatives all have the same type.
pub trait Uniform {
    type Out;
    fn uniform(self) -> Self::Out;
}

/// `Alt` for alternatives that share an item, which `Or` returns as is.
/// ```
/// use takion::*;
///
/// type Sign = Or<(Tok<"+">, Tok<"-">)>;
/// assert_eq!(Sign::parse::<()>("-".into()).must(), "-");
/// ```
pub struct Or<T>(pub T);

impl<T> Rule for Or<T> where Alt<T>: FormatType { type This = Alt<T>; }

impl<'a, U, T> Parse<'a, U> for Or<T>
where
    Alt<T>: Parse<'a, U> + FormatType,
    <Alt<T> as Parse<'a, U>>::Item: Uniform,
{
    const NULLABLE: bool = <Alt<T> as Parse<'a, U>>::NULLABLE;
    type Item = <<Alt<T> as Parse<'a, U>>::Item as Uniform>::Out;

    #[inline]
    fn parse<Cx: Ctx>(cursor: Cursor<'a, U>) -> Ret<'a, U, Self::Item, Cx> {
        let (cur, item) = <Alt<T>>::parse::<Cx>(cursor)??;
        Pass(cur, item.uniform())
    }
}

impl_tuple!{
    { Alt1  A }
    { Alt2  A B }
    { Alt3  A B C }
    { Alt4  A B C D }
    { Alt5  A B C D E }
    { Alt6  A B C D E F }
    { Alt7  A B C D E F G }
    { Alt8  A B C D E F G H }
    { Alt9  A B C D E F G H I }
    { Alt10 A B C D E F G H I J }
    { Alt11 A B C D E F G H I J K }
    { Alt12 A B C D E F G H I J K L }
    { Alt13 A B C D E F G H I J K L M }
    { Alt14 A B C D E F G H I J K L M N }
    { Alt15 A B C D E F G H I J K L M N O }
    { Alt16 A B C D E F G H I J K L M N O P }
    { Alt17 A B C D E F G H I J K L M N O P Q }
    { Alt18 A B C D E F G H I J K L M N O P Q R }
    { Alt19 A B C D E F G H I J K L M N O P Q R S }
    { Alt20 A B C D E F G H I J K L M N O P Q R S T }
    { Alt21 A B C D E F G H I J K L M N O P Q R S T U }
    { Alt22 A B C D E F G H I J K L M N O P Q R S T U V }
    { Alt23 A B C D E F G H I J K L M N O P Q R S T U V W }
    { Alt24 A B C D E F G H I J K L M N O P Q R S T U V W X }
    { Alt25 A B C D E F G H I J K L M N O P Q R S T U V W X Y }
    { Alt26 A B C D E F G H I J K L M N O P Q R S T U V W X Y Z }
}

/// Replaces `$skip` with `$x`, for repeating one type per alternative.
macro same($skip:ident $x:ident) { $x }

macro impl_tuple($({ $ty:ident $($id:ident)* })*) {$(
    impl<'a, Unit, $($id,)+> Parse<'a, Unit> for Alt<($($id,)+)>
    where $($id: Parse<'a, Unit>,)+ {
        const NULLABLE: bool = $($id::NULLABLE)||*;
        type Item = <Alt<($(<$id as Parse<'a, Unit>>::Item,)*)> as Alternation>::Alternate;

        #[inline]
        fn parse<Cx: Ctx>(cursor: Cursor<'a, Unit>) -> Ret<'a, Unit, Self::Item, Cx> {
            $(match $id::parse::<Cx>(cursor) {
                Pass(c, v) => return Pass(c, $ty::$id(v),),
                Miss(_) => {}
                Fail(err) => return Fail(err.wrap::<Self>(cursor.index, cursor.index)),
            })+

            // None of the alternatives matched
            Miss(Cx::Info::new::<Self>(cursor, cursor.index))
        }
    }

    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub enum $ty<$($id),*> { $($id($id),)* }

    impl<$($id,)*> Alternation for Alt<($($id,)*)> { type Alternate = $ty<$($id,)*>; }

    impl<Item> Uniform for $ty<$(same!($id Item)),*> {
        type Out = Item;
        #[inline(always)]
        fn uniform(self) -> Item { match self { $($ty::$id(v))|* => v } }
    }

    impl<$($id: Rule,)*> Rule for Alt<($($id,)*)> {
        type This = Self;
    }

    impl<$($id: Rule,)*> FormatType for Alt<($($id,)*)> {
        fn fmt_type(f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            <Join<(Txt<"${">, Join<($($id,)*), Txt<" | ">>, Txt<"}">)>>::fmt_type(f)
        }
    }

)*}
