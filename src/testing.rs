use std::borrow::Cow;
use crate as takion;

#[doc = include_str!("../README.md")]
#[cfg(doctest)]
pub struct ReadmeDoctests;

use crate::{*};

#[test]
fn destructure() {
    type CurlyNoDestruct<T> = (WsTok<"{">, T, WsTok<"}">);

    assert_eq!(
        <CurlyNoDestruct<Tok<"Hello, World!">>>::parse::<()>(" { Hello, World! } ".into()).must(),
        ("{", "Hello, World!", "}"),
    );

    // note! this is a *type* alias. It also works with arbitrary combinations.
    type Curly<T> = destruct!( (_, body, _) = (WsTok<"{">, T, WsTok<"}">) );

    assert_eq!(
        <Curly<Tok<"Hello, World!">>>::parse::<()>(" { Hello, World! } ".into()).must(),
        "Hello, World!",
    );


    type KeyWord = destruct!( (vis, _, _, body, _) = (Option<WsTok<"pub">>, Tok<"enum">, WsTok<"{">, Punctuate<Ident, WsTok<",">>, WsTok<"}">) );

    assert_eq!(
        <KeyWord>::parse::<()>("pub enum { Hello, World, }".into()).must(),
        (Some("pub"), vec!["Hello", "World"]),
    );
}


#[test]
fn demo() {
    type Curly<T> = destruct!( (_, body, _) = (WsTok<"{">, T, WsTok<"}">) );

    type Parser = (Option<WsTok<"pub">>,
            destruct!( (_, body) = Cut<(WsTok<"enum">, Curly<Punctuate<Ident, WsTok<",">>>)> ));

    println!("===\n{}\n===\n", Parser::type_name());

    dbg!(Parser::display().to_string());

    let input = "pub enum  {  Hello, World, And,everyone_in_it,}";

    assert_eq!(
        Parser::parse::<Traced>(input.into()).must(),
        (Some("pub"), vec![
            "Hello",
            "World",
            "And",
            "everyone_in_it",
        ]),
    );

    let input = "pub enmu  {  Hello, World, And,everyone_in_it,}";

    let res = Parser::parse::<Traced>(input.into()).expect_fail("should have been a fail");
    let expect = r#"While matching for: "pub"? "enum" "{" $(ident)","+ ","? "}"
  at position 0..8
  | pub enmu  {  Hello, World, And,everyone_in_it,}
  | ^^^^^^^^

While matching for: "enum" "{" $(ident)","+ ","? "}"
  at position 4..8
  | pub enmu  {  Hello, World, And,everyone_in_it,}
  |     ^^^^

While matching for: "enum"
  at position 4..8
  | pub enmu  {  Hello, World, And,everyone_in_it,}
  |     ^^^^
"#;
    let formatted = format!("{res}");
    println!("## input\n{input:#?}\n## diagnostic:\n{formatted}");
    assert_eq!(&formatted, expect);
}


#[test]
fn json() {
    #[derive(Debug, Clone, PartialEq)]
    pub enum Json<'a>  {
        Null,
        Boolean(bool),
        String(Cow<'a, str>),
        Number(f64),
        Object(Vec<(Cow<'a, str>, Self)>),
        Array(Vec<Self>),
    }

    type KVPair<T> = (
        QuotedStr<Tok<"\"">, Tok<"\"">, unescape::Full>,
        destruct!((_, v) = Cut<(WsTok<":">, T)>),
    );

    parser!{('a) [Json<'a>] {
        ((body, _): Commit<WsTok<"{">,
                (Option<Intersperse<KVPair<AsRule<Self, Txt<"JSON">>>, WsTok<",">>>,
            WsTok<"}">)>) => Self::Object(body.unwrap_or_default());
        ((body, _): Commit<WsTok<"[">,
                (Option<Intersperse<AsRule<Self, Txt<"JSON">>, WsTok<",">>>,
            WsTok<"]">)>) => Self::Array(body.unwrap_or_default());
        (t: bool) => Self::Boolean(t);
        (n: f64) => Self::Number(n);
        (s: QuotedStr) => Self::String(s);
        ("null") => Self::Null;
    }}

    let input = r#"[
    { "some": "Json", "like": true },
    { "some": -0.8481, "Nothing": null },
    []
]"#;

    assert_eq!(
        Json::parse::<Traced>(input.into()).must(),
        Json::Array(vec![
            Json::Object(vec![
                ("some".into(), Json::String("Json".into())),
                ("like".into(), Json::Boolean(true)),
            ]),
            Json::Object(vec![
                ("some".into(), Json::Number(str::parse::<f64>("-0.8481").unwrap())),
                ("Nothing".into(), Json::Null),
            ]),
            Json::Array(vec![]),
        ])
    );

    let input = r#"{"a": true, "b" }"#;
    let err = Json::parse::<Traced>(input.into()).expect_fail("should have failed");
    let expect = r#"While matching for: $('"'  '"' ":" JSON)","* "}"
  at position 1..17
  | {"a": true, "b" }
  |  ^^^^^^^^^^^^^^^^

While matching for: "," '"'  '"' ":" JSON
  at position 10..17
  | {"a": true, "b" }
  |           ^^^^^^^

While matching for: '"'  '"' ":" JSON
  at position 12..17
  | {"a": true, "b" }
  |             ^^^^^

While matching for: ":" JSON
  at position 15..17
  | {"a": true, "b" }
  |                ^^

While matching for: ":"
  at position 15..17
  | {"a": true, "b" }
  |                ^^
"#;
    let error = format!("{}", err);
    println!("## input\n{input:#?}\n## diagnostic:\n{error}");
    assert_eq!(error, expect);
}
