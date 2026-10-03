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


pub struct Displayed;
impl<T: Display> Prog<T> for Displayed {
    type Output = String;
    fn call(input: T) -> Self::Output { format!("{}", input) }
}

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
