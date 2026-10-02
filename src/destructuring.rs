use crate::*;

/// Keeps the item, in a tuple with the other kept items.
/// ```
/// use takion::*;
/// type Par = Destruct<(Incl, Excl, Incl), (Tok<"Hello">, Tok<", ">, Tok<"World!">)>;
///
/// let (hello, world): (_, _) = Par::parse::<()>("Hello, World!".into()).ok().unwrap();
/// ```
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Incl;

/// Keeps the item as is. It has to be the only kept item.
/// ```
/// use takion::*;
/// type Par = Destruct<(Just, Excl, Excl), (Tok<"Hello">, Tok<", ">, Tok<"World!">)>;
///
/// let hello: &'static str = Par::parse::<()>("Hello, World!".into()).ok().unwrap();
/// assert_eq!(hello, "Hello");
/// ```
///
/// ```compile_fail
/// use takion::*;
/// type Par = Destruct<(Just, Incl), ((Tok<"a">, Tok<"b">), Tok<"c">)>;
///
/// let _ = Par::parse::<()>("abc".into());
/// ```
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Just;

/// Drops the item.
/// ```
/// use takion::*;
/// type Par = Destruct<(Excl, Excl, Excl), (Tok<"Hello">, Tok<", ">, Tok<"World!">)>;
///
/// let _: () = Par::parse::<()>("Hello, World!".into()).ok().unwrap();
/// ```
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Excl;

/// Reshapes the item tuple of `Source` by `Shape`, one of `Incl`, `Just`, `Excl` per item.
/// ```
/// use takion::*;
/// type Par = Destruct<(Incl, Excl, Incl), (Tok<"Hello">, Tok<", ">, Tok<"World!">)>;
///
/// let (hello, world): (_, _) = Par::parse::<()>("Hello, World!".into()).ok().unwrap();
/// ```
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Destruct<Shape, Source>(pub Shape, pub Source);

impl<Shape, Source: Rule> Rule for Destruct<Shape, Source> {
    type This = Source::This;
    type Mod  = Source::Mod;
    type Opt  = Source::Opt;
    type Fmt  = Source::Fmt;
}

impl<'a, T, Shape, Source> Parse<'a, T> for Destruct<Shape, Source>
where
    Source: Parse<'a, T>,
    Shape: Destructure<Source::Item>,
{
    const NULLABLE: bool = Source::NULLABLE;
    type Item = Shape::Output;

    #[inline]
    fn parse<Cx: Ctx>(cursor: Cursor<'a, T>) -> Ret<'a, T, Self::Item, Cx> {
        let (cur, items) = Source::parse::<Cx>(cursor)?
            .map_err(|e| Cx::Info::new::<Self>(cursor, e.end().unwrap_or(cursor.index)))?;
        Pass(cur, Shape::destructure(items))
    }
}

pub struct Only<T>(T);
pub struct Tuple<T>(T);

// TODO: extend these for everything that can use them
#[diagnostic::on_unimplemented(
    message = "`{Self}` can't follow `{Acc}` in a `Destruct` shape",
    note = "`Just` has to be the only kept item, `Incl` keeps several",
)]
pub trait Destructor<Acc> {
    type Wrap<Next>;
    fn wrap<Next>(acc: Acc, next: Next) -> Self::Wrap<Next>;
}

impl<Acc> Destructor<Acc> for Excl {
    type Wrap<Next> = Acc;
    #[inline(always)]
    fn wrap<Next>(acc: Acc, _: Next) -> Self::Wrap<Next> { acc }
}

impl Destructor<()> for Just {
    type Wrap<Next> = Only<Next>;
    #[inline(always)]
    fn wrap<Next>(_: (), next: Next) -> Only<Next> { Only(next) }
}

impl Destructor<()> for Incl {
    type Wrap<Next> = Tuple<(Next,)>;
    #[inline(always)]
    fn wrap<Next>(_: (), next: Next) -> Tuple<(Next,)> { Tuple((next,)) }
}

pub trait Finish {
    type Out;
    fn finish(self) -> Self::Out;
}

impl Finish for () {
    type Out = ();
    #[inline(always)]
    fn finish(self) {}
}

impl<T> Finish for Only<T> {
    type Out = T;
    #[inline(always)]
    fn finish(self) -> T { self.0 }
}

impl<T> Finish for Tuple<T> {
    type Out = T;
    #[inline(always)]
    fn finish(self) -> T { self.0 }
}


pub trait Destructure<Items> {
    type Output;
    fn destructure(items: Items) -> Self::Output;
}

impl Destructure<()> for () {
    type Output = ();
    #[inline(always)]
    fn destructure((): ()) {}
}

macro_rules! impl_des_inner {
    ($($a:ident)+) => {
        #[allow(non_snake_case)]
        impl<$($a,)+> Destructor<Tuple<($($a,)+)>> for Incl {
            type Wrap<Next> = Tuple<($($a,)+ Next,)>;
            #[inline(always)]
            fn wrap<Next>(Tuple(($($a,)+)): Tuple<($($a,)+)>, next: Next) -> Self::Wrap<Next> {
                Tuple(($($a,)+ next,))
            }
        }
    };
}

macro_rules! impl_des {
    () => {};
    ($z:ident $($a:tt)*) => {
        impl_des_inner! { $z $($a)* }
        impl_des! { $($a)* }
    };
}

impl_des! { A B C D E F G H I J K L M N O P Q R S T U V W X Y Z }

macro_rules! impl_destructure {
    (@rec [$($done:tt)*] $Acc:tt) => {};

    (@rec [$($done:tt)*] $Acc:tt $S:ident $P:ident $A:ident $($rest:tt)*) => {
        impl_destructure! { @emit [$($done)* ($S, $Acc, $P, $A)] $A }
        impl_destructure! { @rec  [$($done)* ($S, $Acc, $P, $A),] $A $($rest)* }
    };

    (@emit [$(($S:ident, $AccIn:tt, $P:ident, $A:ident)),+ $(,)?] $Final:ident) => {
        #[allow(non_snake_case)]
        impl<$($S, $P, $A,)+> Destructure<($($P,)+)> for ($($S,)+)
        where
            $($S: Destructor<$AccIn, Wrap<$P> = $A>,)+
            $Final: Finish,
        {
            type Output = <$Final as Finish>::Out;

            #[inline(always)]
            fn destructure(($($P,)+): ($($P,)+)) -> Self::Output {
                let acc = ();
                $(let acc = $S::wrap(acc, $P);)+
                acc.finish()
            }
        }
    };

    ($($t:ident)*) => { impl_destructure! { @rec [] () $($t)* } };
}

impl_destructure! {
    S1  P1  A1  S2  P2  A2  S3  P3  A3  S4  P4  A4  S5  P5  A5
    S6  P6  A6  S7  P7  A7  S8  P8  A8  S9  P9  A9  S10 P10 A10
    S11 P11 A11 S12 P12 A12 S13 P13 A13 S14 P14 A14 S15 P15 A15
    S16 P16 A16 S17 P17 A17 S18 P18 A18 S19 P19 A19 S20 P20 A20
    S21 P21 A21 S22 P22 A22 S23 P23 A23 S24 P24 A24 S25 P25 A25
    S26 P26 A26
}
