#![feature(decl_macro)]
use std::env::args;

use takion::{Range::Exactly, strings::Basic, syntax::RangesP, *};

const HELP: &str = r#"string-expansion: expands shell-like string templates

usage: string-expansion <template>...
       (multiple arguments are joined with spaces)

patterns:
  {a|b|c}    alternatives
  {0..3}     range, exclusive:  0 1 2
  {0..=3}    range, inclusive:  0 1 2 3
  \{ \} \|   literal characters

example:
  $ string-expansion '{a|b}_{0..2}'
  a_0
  a_1
  b_0
  b_1

  $ string-expansion '\{{a|b}\}_\{{0..2}\}'
  {a}_{0}
  {a}_{1}
  {b}_{0}
  {b}_{1}
"#;

// Raw keeps escapes intact; Seek unescapes.
type Raw<T> = UTF8<Until<T, (Option<Tok<"\\">>, Next)>>;
type Seek<T> = Map<Pipe<Raw<T>, Unescape<Basic>>, Coax<String>>;

type Sep = Alt<(Tok<"|">, End)>;

// The brace body is captured raw so `\|` is still escaped when Pattern splits on `|`.
type Pat = Destruct<(Just, Excl),
    (Commit<Tok<"{">, (Pipe<Raw<Tok<"}">>, Pattern>, Tok<"}">)>)>;


struct Pattern;

parser! {[Pattern] -> [Vec<String>] {
    (range: RangesP<u64, Tok<"..">, Tok<"..=">>) => range
        .may_enumerate()
        .expect("must be a definitive range")
        .into_iter().map(|v| format!("{v}"))
        .collect();
    (parts: Intersperse<Seek<Sep>, Tok<"|">>) => parts;
}}

struct Template;

parser! {[Template] -> [Vec<String>] {
    (parts: Vec<Or<(Pat, Repeat<Seek<Alt<(Tok<"{">, End)>>, {count![1]}>)>>) => {
        let parts: Vec<Vec<String>> = parts;
        // Cartesian product; the leftmost pattern varies slowest.
        parts.into_iter().fold(vec![String::new()], |acc, choices| {
            acc.iter()
                .flat_map(|prefix| choices.iter().map(move |c| format!("{prefix}{c}")))
                .collect()
        })
    };
}}


fn main() {
    let args = args().skip(1).collect::<Vec<_>>();
    if matches!(args.first().map(String::as_str), None | Some("-h" | "--help")) {
        print!("{HELP}");
        return;
    }

    let input = args.join(" ");
    Template::parse::<Traced>(input.as_str().into())
        .must()
        .into_iter()
        .for_each(|v| println!("{v}"))
}

test_cases! {
    "{a|b}_{0..2}" => ["a_0", "a_1", "b_0", "b_1"];
    "plain" => ["plain"];
    "x{1..=3}" => ["x1", "x2", "x3"];
    "pre-{a|b}-post" => ["pre-a-post", "pre-b-post"];
    "{a|b}{c|d}" => ["ac", "ad", "bc", "bd"];
    "{0..2}{0..2}" => ["00", "01", "10", "11"];
    r"\{a\}" => ["{a}"];
    r"{a\|b|c}" => ["a|b", "c"];
}

macro test_cases($( $input:literal => $expect:expr ;)*) {
    #[test]
    fn string_expansion() {$(
        assert_eq!(
            Template::parse::<Traced>($input.into()).must(),
            &$expect,
        );
    )*}
}
