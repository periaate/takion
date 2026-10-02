//! Escape decoding. `Unescape<Domain>` decodes the escapes `Domain` knows,
//! and borrows the input where there's nothing to decode.
use super::*;
use std::borrow::Cow;

/// `\n`, `\r`, `\t`, `\0`, `\\`, `\"`, `\'`, and any other `\x` as `x`.
pub struct Basic;

parser! { [Basic] -> [char] {
    ("\\n") => '\n';
    ("\\r") => '\r';
    ("\\t") => '\t';
    ("\\0") => '\0';
    ("\\\\") => '\\';
    ("\\\"") => '\"';
    ("\\\'") => '\'';

    (_: Tok<"\\">, c: Unit) => c as char;
    // TODO: add `char` impl that parses the next char, not just u8.
    // (_: Tok<"\\">, c: char) => c as char;
}}

/// `\u0041` and `\u{41}`.
pub struct Unicode;

parser! { [Unicode] -> [char] {
    ((_, hex): (Tok<"\\u">, UTF8<Slice<Hex<{Case::Either}>, {count![4]}>>)) => decode(hex);
    ((_, hex, _): (Tok<"\\u{">, UTF8<Slice<Hex<{Case::Either}>, {count![1..=6]}>>, Tok<"}">)) => decode(hex);
}}

fn decode(hex: &str) -> char {
    u32::from_str_radix(hex, 16).ok().and_then(char::from_u32).unwrap_or('\u{FFFD}')
}

/// `Unicode` and `Basic`.
pub type Full = Or<(Unicode, Basic)>;

/// A run of input without escapes.
type Plain = ToCow<UTF8<Until<Alt<(Tok<"\\">, End)>>>>;

/// Decodes the escapes of `Domain`.
/// ```
/// use takion::*;
///
/// assert_eq!(<Unescape<unescape::Full>>::parse::<()>(r"\u0041\n".into()).must(), "A\n");
/// ```
pub struct Unescape<Domain>(pub Domain);

impl<D: Rule> Rule for Unescape<D> { type This = Txt<"">; }

impl<'a, D> Parse<'a, u8> for Unescape<D>
where D: Parse<'a, u8>, D::Item: IntoCow<'a>,
{
    type Item = Cow<'a, str>;

    fn parse<Cx: Ctx>(cursor: Cursor<'a, u8>) -> Ret<'a, u8, Self::Item, Cx> {
        if cursor.is_empty() { return Pass(cursor, Cow::Borrowed("")) }

        let (cur, mut frags) = <Repeat<Or<(ToCow<D>, Plain)>>>::parse::<Cx>(cursor)??;
        if frags.len() == 1 && let Some(frag) = frags.pop() { return Pass(cur, frag) }
        Pass(cur, Cow::Owned(frags.concat()))
    }
}

#[test]
fn unescapes() {
    let basic = vec![
        (r"\n", "\n"),
        (r"\t", "\t"),
        (r"\0", "\0"),
        (r"\r", "\r"),
        (r"Hello\nWorld", "Hello\nWorld"),
        (r"Column1\tColumn2", "Column1\tColumn2"),
        (r"C:\\Program Files", r#"C:\Program Files"#),
    ];

    for (input, expect) in basic {
        assert_eq!(
            <Unescape<Basic>>::parse::<Traced>(input.into()).must(),
            expect,
        );
    }

    let unicode = vec![
        (r"Alpha: \u03B1", "Alpha: \u{03B1}"),
        (r"Heart: \u2665", "Heart: \u{2665}"),
        (r"Sparkles: \u{2728}", "Sparkles: \u{2728}"),
        (r"Crab: \u{1F980}", "Crab: \u{1F980}"),
        (r"Short: \u{A}", "Short: \u{A}"),
    ];

    for (input, expect) in unicode {
        assert_eq!(
            <Unescape<Unicode>>::parse::<Traced>(input.into()).must(),
            expect,
        );
    }

    let mixed = vec![
        (r"A\nB\u0041", "A\nBA"),
        (r"plain", "plain"),
        (r"", ""),
    ];
    for (input, expect) in mixed {
        assert_eq!(
            <Unescape<Full>>::parse::<Traced>(input.into()).must(),
            expect,
        );
    }
}
