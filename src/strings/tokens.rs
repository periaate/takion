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

/// Unordered choice between literals: the longest one that matches wins, so the order
/// they are listed in never matters. Listing a literal twice (or an empty one) is
/// ambiguous, and fails to compile.
/// ```
/// use takion::*;
///
/// type Cmp = Choose<{ &["<", "<<=", "<=", "<<"] }>;
/// assert_eq!(Cmp::parse::<()>("<<= 1".into()).must(), "<<=");
/// assert_eq!(Cmp::parse::<()>("<<1".into()).must(),   "<<");
/// assert_eq!(Cmp::parse::<()>("<=1".into()).must(),   "<=");
/// assert_eq!(Cmp::parse::<()>("< 1".into()).must(),   "<");
/// assert!(Cmp::parse::<()>("> 1".into()).is_miss());
/// ```
pub struct Choose<const CHOICES: &'static [&'static str]>;

/// Evaluated at compile time for every `Choose`: panics if two choices are the same.
const fn unambiguous(choices: &[&str]) {
    let mut i = 0;
    while i < choices.len() {
        assert!(!choices[i].is_empty(), "Choose: an empty literal matches everywhere");
        let mut j = i + 1;
        while j < choices.len() {
            assert!(!same(choices[i].as_bytes(), choices[j].as_bytes()), "Choose: a literal is listed twice");
            j += 1;
        }
        i += 1;
    }
}

const fn same(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() { return false }
    let mut i = 0;
    while i < a.len() {
        if a[i] != b[i] { return false }
        i += 1;
    }
    true
}

impl<const C: &'static [&'static str]> Rule for Choose<C> { type This = Self; }

impl<const C: &'static [&'static str]> FormatType for Choose<C> {
    fn fmt_type(f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("${")?;
        for (i, choice) in C.iter().enumerate() {
            if i > 0 { f.write_str(" | ")? }
            write!(f, "\"{choice}\"")?;
        }
        f.write_str("}")
    }
}

impl<'a, const C: &'static [&'static str]> Parse<'a, u8> for Choose<C> {
    type Item = &'static str;

    #[inline]
    fn parse<Cx: Ctx>(cursor: Cursor<'a, u8>) -> Ret<'a, u8, Self::Item, Cx> {
        const { unambiguous(C) }
        let rest = cursor.rest();
        let longest = C.iter().copied()
            .filter(|choice| rest.starts_with(choice.as_bytes()))
            .max_by_key(|choice| choice.len());

        match longest {
            Some(choice) => Pass(cursor.advance(choice.len()), choice),
            None => Miss(Cx::Info::new::<Self>(cursor, cursor.index)),
        }
    }
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
