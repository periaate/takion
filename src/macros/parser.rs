use crate::*;

/// QoL macro for parsers only composed of other parsers.
/// ```
/// use takion::*;
///
/// #[derive(Debug, Clone, Copy, PartialEq)]
/// pub enum Step {
///     Limit(u32),
///     Reduce(u32),
///     Crop(u32),
///     Pad(u32, u32),
///
///     HFlip,
///     VFlip,
///
///     Rotate(f32),
///
///     Blur(f32),
///     Sharpen(f32, i32),
///     Opacity(f32),
///
///     Gray,
///     Invert,
/// }
/// 
/// type Wrapped<F, T> =
///    Destruct<(Excl, Just), 
///    (F, Cut<Destruct<(Excl, Just, Excl), (Tok<"(">, T, Tok<")">)>>)>;
///
/// 
/// parser!{ [ Step ] {
///     ("hflip")  => Step::HFlip;
///     ("vflip")  => Step::VFlip;
///     ("gray")   => Step::Gray;
///     ("invert") => Step::Invert;
///
///     (_: Tok<"limit">,   _: Tok<"(">, val: u32, _: Tok<")">) => Step::Limit  (val);
///     (_: Tok<"crop">,    _: Tok<"(">, val: u32, _: Tok<")">) => Step::Crop   (val);
///     (_: Tok<"blur">,    _: Tok<"(">, val: f32, _: Tok<")">) => Step::Blur   (val);
///     (_: Tok<"opacity">, _: Tok<"(">, val: f32, _: Tok<")">) => Step::Opacity(val);
///     
///     // or we can use the type alias we defined earlier!
///     (val: Wrapped<Tok<"reduce">, u32>) => Step::Reduce(val);
///     (val: Wrapped<Tok<"rotate">, f32>) => Step::Rotate(val);
///
///     ((w, _, h): Wrapped<Tok<"pad">, (u32, Tok<",">, u32)>) => Step::Pad(w, h);
///     ((s, _, t): Wrapped<Tok<"sharpen">, (f32, Tok<",">, i32)>) => Step::Sharpen(s, t);
/// }}
/// ```
///
/// Note:
/// `parser` does *not* compute nullability bounds based on the grammar shape.
/// These create circular dependencies too easily. Explicit controls for this in
/// the parser macro will be added later.
pub macro parser {
    // If parsers Item is absent, default to `Self`
    (($lt:lifetime $(, $($gen:tt)*)? ) [ $Type:ty ] $body:tt) => {
        parser!{ ($lt $(, $($gen:tt)*)? ) [$Type] -> [$Type] $body }
    },

    (($lt:lifetime $(, $($gen:tt)*)? ) [ $Type:ty ] -> [ $Into:ty ]
        { $( $arg:tt => $Body:expr; )+ }) => {
        parser!{ ($lt $(, $($gen:tt)*)? ) [$Type] -> [$Into]
            Rule { Alt<($( type_pair!( $arg ) ),* )> }
            { $( $arg => $Body; )+ } }
    },

    (($lt:lifetime) [ $Type:ty ] -> [ $Into:ty ] Rule{ $rule:ty } { $( $arg:tt => $Body:expr ; )+ }) => {

        impl<$lt> Rule for $Type { type This = $rule; }

        impl<$lt> Parse<$lt, u8> for $Type {
            type Item = $Into;
            // This creates circular dependencies a bit too easily at this point.
            // const NULLABLE: bool = $(<type_pair!($($args)*) as Parse<$lt, u8>>::NULLABLE)||*;
            // const NULLABLE: bool = $(tnull!($($args)*))||*;

            #[inline]
            fn parse<Cx: Ctx>(cursor: Cursor<$lt, u8>) -> Ret<$lt, u8, Self::Item, Cx> {
                $(parser_pair!([Cx] [cursor]
                    $arg => $Body
                );)+
                Miss(Cx::Info::new::<Self>(cursor, cursor.index))
            }
        }
    },

    // If parsers lifetime argument is absent, inject `('a)`
    ([ $Type:ty ] $($rest:tt)*) => {
        parser!{('a) [$Type] $($rest)* }
    },
}

pub mod internals {
    use super::*;
    pub macro parser_pair {
        ([$Cx:ident] [$cursor:ident] { $($any:tt)* } => $Body:expr ) => {
            if let SubRet::Pass(c, _) = <crate::pat!{$($any)*}>::parse::<$Cx>($cursor)? {
                return Pass(c, $Body)
            }
        },
        ([$Cx:ident] [$cursor:ident] ( $( $($ty:literal)|+ ),* $(,)? ) => $Body:expr ) => {
            if let SubRet::Pass(c, _) = <( $( Alt<($(Tok<$ty>,)+)> ),* )>::parse::<$Cx>($cursor)? {
                return Pass(c, $Body)
            }
        },
        ([$Cx:ident] [$cursor:ident] ( $( $id:tt : $ty:ty ),* $(,)? ) => $Body:expr ) => {
            if let SubRet::Pass(c, ( $( $id ,)* )) = <( $( $ty ,)* )>::parse::<$Cx>($cursor)? {
                return Pass(c, $Body)
            }
        },
        ([$Cx:ident] [$cursor:ident] ( $( $ty:ty ),* $(,)? ) => $Body:expr ) => {
            if let SubRet::Pass(c, _) = <( $( $ty, )* )>::parse::<$Cx>($cursor)? {
                return Pass(c, $Body)
            }
        },
    }

    pub macro type_pair {
        ( {$($rest:tt)*} )           => { pat!($($rest)*) },
        ( ($( $($ty:literal)|+ ),* $(,)? ) ) => { ( $( Alt<($(Tok<$ty>,)+)> ,)* ) },
        ( ($( $id:tt : $ty:ty ),* $(,)? ) )  => { ( $( $ty ,)* ) },
        ( ($( $ty:ty ),* $(,)? ) )           => { ( $( $ty ,)* ) },
    }
}

/// Tries each arm in order. On the first `Pass`, returns `Pass(c, body)` from
/// the enclosing function. Falls through if nothing matches, so the caller
/// decides what a miss looks like.
/// ```ignore
/// fn parse<Cx: Ctx>(cursor: Cursor<'a, u8>) -> Ret<'a, u8, Self::Item, Cx> {
///     parse!{ [Cx] [cursor] {
///         (a: u32, _: Tok<",">, b: u32) => Foo(a, b);
///         ("none" | "null")             => Foo(0, 0);
///     }}
///     Miss(Cx::Info::new::<Self>(cursor, cursor.index))
/// }
/// ```
pub macro parse([$Cx:ident] [$cursor:ident] { $( $arg:tt => $Body:expr; )* }) {
    $( parser_pair!([$Cx] [$cursor] $arg => $Body); )+
}

pub(crate) use internals::{parser_pair, type_pair};
