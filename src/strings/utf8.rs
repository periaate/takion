use super::*;
use std::borrow::Cow;

/// Matches `P` and returns its bytes as `&str`. Invalid UTF-8 is a `Fail`.
/// ```
/// use takion::*;
///
/// type Word = UTF8<Slice<Between<b'a', b'z'>>>;
/// assert_eq!(Word::parse::<()>("abc1".into()).must(), "abc");
/// ```
#[derive(Clone, Copy, PartialEq, Eq)]
 pub struct UTF8<P>(pub P);

impl<P: Rule> Rule for UTF8<P> { type This = P::This; }

impl<'a, T, P: Parse<'a, T, Item = &'a [u8]>> Parse<'a, T> for UTF8<P> {
    type Item = &'a str;

    #[inline]
    fn parse<Cx: Ctx>(cursor: Cursor<'a, T>) -> Ret<'a, T, Self::Item, Cx> {
        let (cur, out) = <P>::parse::<Cx>(cursor)
            .map_err(|e| e.wrap::<Self>(cursor.index, cursor.index))??;
        let Ok(res) = str::from_utf8(out) else {
            return Fail(Cx::Info::new::<Self>(cursor, cur.index))
        };
        Ret::Pass(cur, res)
    }
}


pub trait Prog<I> {
    type Output;
    fn call(input: I) -> Self::Output;
}

pub struct Map<A, B>(pub A, pub B);
pub struct Coax<T: ?Sized>(PhantomData<T>);

macro coax([$lt:lifetime]($input:ident); $([$T:ty] [$I:ty] -> [$O:ty] $body:tt;)*) {$(
impl<$lt> Prog<$I> for Coax<$T> {
    type Output = $O;
    fn call($input: $I) -> Self::Output $body
}
)*}

coax!{['a] (input);
    // [str] [&'a [u8]] -> [Option<&'a str>]
    //     { str::from_utf8(input).ok() };
    // [&'a str] [&'a [u8]] -> [Option<&'a str>]
    //     { <Coax<str>>::call(input) };
    [Cow<'a, str>] [Cow<'a, str>] -> [Cow<'a, str>] {input};
    // [Cow<'a, str>] [&'a [u8]] -> [Option<Cow<'a, str>>]
    //     { <Coax<Cow<'a, str>>>::call(<Coax<str>>::call(input)) };
    [Cow<'a, str>] [&'a str] -> [Cow<'a, str>]
        { Cow::Borrowed(input.into()) };
    [Cow<'a, str>] [char] -> [Cow<'a, str>]
        { Cow::Owned(input.into()) };
    // [String] [&'a [u8]] -> [Option<String>]
    //     { <Coax<str>>::call(input).map(ToString::to_string) };
    [String] [&'a str] -> [String]
        { input.to_string() };
    [String] [Cow<'a, str>] -> [String]
        { input.to_string() };
}

impl<P: Rule, F> Rule for Map<P, F> { type This = P::This; }


impl<'a, T, P: Parse<'a, T>, F: Prog<P::Item>> Parse<'a, T> for Map<P, F> {
    type Item = F::Output;
    fn parse<Cx: Ctx>(cursor: Cursor<'a, T>) -> Ret<'a, T, Self::Item, Cx>
        { P::parse(cursor).map(F::call) }
}

pub struct Displayed;
impl<T: Display> Prog<T> for Displayed {
    type Output = String;
    fn call(input: T) -> Self::Output { format!("{}", input) }
}

pub type Cast<P, T> = Map<P, Coax<T>>;
pub type Stringify<P> = Cast<P, String>;
pub struct ToCow<P>(pub P);

impl<P: Rule> Rule for ToCow<P> { type This = P::This; }

impl<'a, T, P: Parse<'a, T>> Parse<'a, T> for ToCow<P>
where P::Item: IntoCow<'a> {
    type Item = Cow<'a, str>;
    fn parse<Cx: Ctx>(cursor: Cursor<'a, T>) -> Ret<'a, T, Self::Item, Cx>
        { P::parse(cursor).map(IntoCow::into_cow) }
}

/// Items that become a `Cow<str>`, borrowing where they can.                                    
pub trait IntoCow<'a> { fn into_cow(self) -> Cow<'a, str>; }                                     
impl<'a> IntoCow<'a> for &'a str { fn into_cow(self) -> Cow<'a, str> { Cow::Borrowed(self) } }   
impl<'a> IntoCow<'a> for char    { fn into_cow(self) -> Cow<'a, str> { Cow::Owned(self.into()) } }
