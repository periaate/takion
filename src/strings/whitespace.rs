use super::*;

/// Tok which skips whitespace both before and after the token.
/// ```
/// use takion::*;
///
/// let (cursor, _) = WsTok::<"let">::parse::<()>("   let   x".into()).unwrap().unwrap();
/// assert_eq!(std::str::from_utf8(cursor.rest()).unwrap(), "x");
/// ```
pub type WsTok<const S: &'static str> =
    Destruct<(Excl, Just, Excl), (SkipWs, Tok<S>, SkipWs)>;

/// The most common whitespace characters.
pub type WhiteSpace = AsRule<Span<pat!{ " "|"\r"|"\t"|"\n" }>, ()>;

/// More than one whitesaces.
pub type Ws = AsRule<Slice<WhiteSpace>, ()>;

/// Any number of whitespaces.
pub type SkipWs = AsRule<Option<Ws>, ()>;
