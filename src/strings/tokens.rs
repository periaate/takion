use super::*;

/// Tok is a literal string matcher. Only matches exactly what its type says
/// ```
/// use takion::{Parse, Tok};
///
/// type TakionParser = Tok<"takion">;
///
/// assert_eq!("takion", TakionParser::parse::<()>("takion".into()).ok().unwrap());
/// ```
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Tok<const S: &'static str>;

impl<const S: &'static str> AsRef<str> for Tok<S> {
    fn as_ref(&self) -> &str { S }
}

impl<const A: &'static str> Rule for Tok<A> {
    type This = Join<(Txt<"\"">, Txt<A>, Txt<"\"">)>;
}

/// Between is the inclusive range (similar to regex) between some two charaters.
/// E.g., `Digit` is defined as: `pub type Digit = Between<b'0', b'9'>;`
/// ```rs
/// use takion::{Parse, Between, Exactly};
/// type HexLowerCase = Alt<(Between<b'a', b'f'>, Between<b'0', b'9'>)>;
/// type RGB = (Tok<"#">, Exactly<6, HexLowerCase>);
///
/// assert!(RGB::parse::<()>("#0c953a".into()).is_pass());
/// assert!(RGB::parse::<()>("0c953a".into()).is_miss());
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Between<const START: u8, const END: u8>;

impl<const FROM: u8, const UPTO: u8> Rule for Between<FROM, UPTO> {
    type This = Join<(Ascii<FROM>, Txt<"..=">, Ascii<UPTO>)>;
}

impl<'a, const START: u8, const END: u8> Parse<'a, u8> for Between<START, END> {
    type Item = u8;

    #[inline]
    fn parse<Cx: Ctx>(cursor: Cursor<'a, u8>) -> Ret<'a, u8, Self::Item, Cx> {
        if let Some((cur, res)) = cursor.take_one().map(|(c, v)| (c, *v))
        && START <= res && res <= END {
            return Pass(cur, res)
        }
        Miss(Cx::Info::new::<Self>(cursor, cursor.index))
    }
}


impl<'a, const S: &'static str> Parse<'a, u8> for Tok<S> {
    const NULLABLE: bool = S.is_empty();
    type Item = &'static str;

    #[inline]
    fn parse<Cx: Ctx>(cursor: Cursor<'a, u8>) -> Ret<'a, u8, Self::Item, Cx> {
        let byt = const { S.as_bytes() };
        let len = const { S.len() };
        if let Some((cur, val)) = cursor.may_slice(len) && val == byt { return Pass(cur, S) }
        Miss(Cx::Info::new::<Self>(cursor, cursor.source.len().min(cursor.index + len)))
    }
}

#[test]
fn tok_display_debug() {
    assert_eq!(format!("{}", Tok::<"Hello, World!">), "Hello, World!");
    assert_eq!(format!("{:?}", Tok::<"Hello, World!">), "Tok<\"Hello, World!\">");
}

impl <const S: &'static str>std::fmt::Debug for Tok<S>{
    fn fmt(&self,f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(std::format_args!("Tok<{S:?}>"))
    }
}

impl <const S: &'static str>std::fmt::Display for Tok<S>{
    fn fmt(&self,f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(S)
    }
}

/// ```
/// use takion::*;
///
/// assert!(Digit::parse::<()>("7".into()).is_pass());
/// assert!(Digit::parse::<()>("a".into()).is_miss());
/// ```
pub type Digit = Between<b'0', b'9'>;

impl<const FROM: u8, const UPTO: u8> Literal<str> for Between<FROM, UPTO> {
    fn literal() -> &'static str {
        let lit = const { match str::from_utf8(&[FROM, b'.', b'.', b'=', UPTO]) {
            Ok(v) => v,
            Err(e) => panic!("Between is not valid utf8"),
        } };
        lit
    }
}

/// ```
/// use takion::*;
///
/// assert!(Letter::parse::<()>("x".into()).is_pass());
/// assert!(Letter::parse::<()>("9".into()).is_miss());
/// ```
pub type Letter = Span<Alt<(Between<b'A', b'Z'>, Between<b'a', b'z'>)>>;
