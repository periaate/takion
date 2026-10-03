use super::*;

use std::borrow::Cow;


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
pub type Cast<P, T> = Map<P, Coax<T>>;
