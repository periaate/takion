#![allow(incomplete_features)]
#![feature(decl_macro, adt_const_params, unsized_const_params)]
use std::env::args;

use takion::*;

const HELP: &str = r#"expressions: parses and evaluates expressions

usage: expressions [lisp] <expression>...

example:
  $ expressions '1 < 2 ? 10 + 1 : 20'
  (Cond (Lt 1 2) (Add 10 1) 20)
  = 11

  $ expressions lisp '(max 1 (* 2 3) 4)'
  (Max 1 (Mul 2 3) 4)
  = 6
"#;

fn main() {
    let args = args().skip(1).collect::<Vec<_>>();
    let (lisp, rest) = match args.split_first() {
        Some((first, rest)) if first == "lisp" => (true, rest),
        _ => (false, args.as_slice()),
    };
    let src = rest.join(" ");
    if src.is_empty() || src == "-h" || src == "--help" { return print!("{HELP}") }

    if lisp {
        let tree = <UseAll<Expr<f64, Lisp>>>::parse::<Traced>(src.as_str().into()).must();
        println!("{tree}\n= {}", run(&tree));
    } else {
        let tree = <UseAll<Expr<f64, Js>>>::parse::<Traced>(src.as_str().into()).must();
        println!("{tree}\n= {}", eval(&tree));
    }
}

/// One row per input: the tree it parses to, and what it evaluates to.
macro table($test:ident [$Op:ty] [$eval:ident] { $($input:literal => $tree:literal = $value:literal;)* }) {
    #[test]
    fn $test() {$(
        let tree = <UseAll<Expr<f64, $Op>>>::parse::<Traced>($input.into()).must();
        assert_eq!(tree.to_string(), $tree, "tree of {:?}", $input);
        assert_eq!($eval(&tree), $value, "value of {:?}", $input);
    )*}
}

fn num(b: bool) -> f64 { f64::from(u8::from(b)) }


/* ====| infix: math, comparison, logic, and a ternary |==== */

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Js {
    Fact, Pow, Neg, Not, Mul, Div, Mod, Add, Sub,
    Lt, Le, Gt, Ge, Eq, Ne, And, Or, Cond,
}

operators!{ ('a, E) [Js] {
    { Out((e, _): Commit<WsTok<"(">, (E, WsTok<")">)>) => e; }
    { Suf("!") => Self::Fact; }
    right { In("^") => Self::Pow; }
    { Pre("-") => Self::Neg; Pre("!") => Self::Not; }
    { In("*") => Self::Mul; In("/") => Self::Div; In("%") => Self::Mod; }
    { In("+") => Self::Add; In("-") => Self::Sub; }
    { In("<=") => Self::Le; In("<") => Self::Lt; In(">=") => Self::Ge; In(">") => Self::Gt; }
    { In("==" | "===") => Self::Eq; In("!=" | "!==") => Self::Ne; }
    { In("&&") => Self::And; }
    { In("||") => Self::Or; }
    // `? .. :` is one infix operator with an expression inside it: the middle operand.
    right { In((mid, _): Commit<WsTok<"?">, (E, WsTok<":">)>) => (Self::Cond, mid); }
}}

/// A chain is one list, and it is read here: `-` and `/` fold left, `^` folds right,
/// comparisons hold between neighbours (`1 < 2 < 3`). Any length works for any operator.
fn eval(tree: &Tree<f64, Js>) -> f64 {
    use Js::*;
    match tree {
        Tree::Term(n) => *n,
        Tree::List(op, operands) => {
            let v = operands.iter().map(eval).collect::<Vec<_>>();
            let between = |holds: fn(f64, f64) -> bool| num(v.windows(2).all(|w| holds(w[0], w[1])));
            match op {
                Add  => v.iter().sum(),
                Mul  => v.iter().product(),
                Sub  => match v.split_first() {
                    Some((x, [])) => -x,
                    Some((x, rest)) => x - rest.iter().sum::<f64>(),
                    None => 0.0,
                },
                Div  => v.iter().copied().reduce(|a, b| a / b).unwrap_or(1.0),
                Mod  => v.iter().copied().reduce(|a, b| a % b).unwrap_or(0.0),
                Pow  => v.iter().copied().rev().reduce(|acc, x| x.powf(acc)).unwrap_or(1.0),
                Neg  => -v.iter().sum::<f64>(),
                Fact => v.iter().map(|&n| (1..=n as u64).product::<u64>() as f64).product(),
                Not  => num(v.iter().all(|&x| x == 0.0)),
                And  => num(v.iter().all(|&x| x != 0.0)),
                Or   => num(v.iter().any(|&x| x != 0.0)),
                Lt   => between(|a, b| a <  b),
                Le   => between(|a, b| a <= b),
                Gt   => between(|a, b| a >  b),
                Ge   => between(|a, b| a >= b),
                Eq   => between(|a, b| a == b),
                Ne   => between(|a, b| a != b),
                Cond => if v[0] != 0.0 { v[1] } else { v[2] },
            }
        }
    }
}

type Calc = Expr<f64, Js>;

table! { javascript [Js] [eval] {
    // Arithmetic and chaining
    "1 + 2 * 3"          => "(Add 1 (Mul 2 3))"                       = 7.0;
    "10 % 4 * 2"         => "(Mul (Mod 10 4) 2)"                      = 4.0;
    "1+2*3-4/2"          => "(Sub (Add 1 (Mul 2 3)) (Div 4 2))"       = 5.0;
    "1 - 2 - 3"          => "(Sub 1 2 3)"                             = -4.0;
    "1 - 2 + 3"          => "(Add (Sub 1 2) 3)"                       = 2.0;
    "1 - (2 - 3)"        => "(Sub 1 (Sub 2 3))"                       = 2.0;
    "8 / 2 / 2"          => "(Div 8 2 2)"                             = 2.0;
    "2 ^ 3 ^ 2"          => "(Pow 2 3 2)"                             = 512.0;
    "(2 ^ 3) ^ 2"        => "(Pow (Pow 2 3) 2)"                       = 64.0;

    // Other math
    "-2 ^ 2"             => "(Neg (Pow 2 2))"                         = -4.0;
    "2 ^ -2"             => "(Pow 2 (Neg 2))"                         = 0.25;
    "- - 3"              => "(Neg (Neg 3))"                           = 3.0;
    "-3!"                => "(Neg (Fact 3))"                          = -6.0;
    "2 * 3! + 1"         => "(Add (Mul 2 (Fact 3)) 1)"                = 13.0;
    "!0 + 1"             => "(Add (Not 0) 1)"                         = 2.0;

    // Comparators
    "1 < 2 < 3"          => "(Lt 1 2 3)"                              = 1.0;
    "3 > 2 > 2"          => "(Gt 3 2 2)"                              = 0.0;
    "1 + 1 == 2 && 2 * 2 != 5" => "(And (Eq (Add 1 1) 2) (Ne (Mul 2 2) 5))" = 1.0;
    "0 || 0 && 1"        => "(Or 0 (And 0 1))"                        = 0.0;

    // Choose | Unordered Choice
    "3! != 6"            => "(Ne (Fact 3) 6)"                         = 0.0;
    "3!=6"               => "(Ne 3 6)"                                = 1.0;
    "1<=2"               => "(Le 1 2)"                                = 1.0;
    "1 === 1"            => "(Eq 1 1)"                                = 1.0;
    "1 !== 2"            => "(Ne 1 2)"                                = 1.0;

    // ternary
    "1 < 2 ? 10 : 20"    => "(Cond (Lt 1 2) 10 20)"                   = 10.0;
    "0 ? 1 : 0 ? 2 : 3"  => "(Cond 0 1 (Cond 0 2 3))"                 = 3.0;
    "1 ? 0 ? 1 : 2 : 3"  => "(Cond 1 (Cond 0 1 2) 3)"                 = 2.0;
    "1 ? 2 + 3 : 4 || 5" => "(Cond 1 (Add 2 3) (Or 4 5))"             = 5.0;
    "1 + 1 ? 5 : 6"      => "(Cond (Add 1 1) 5 6)"                    = 5.0;

    "  ( 1 + 2 )  *  3"  => "(Mul (Add 1 2) 3)"                       = 9.0;
}}

#[test]
fn boundaries() {
    let rest = |s: &'static str| {
        let (cursor, _) = Calc::parse::<()>(s.into()).option().expect("should pass");
        std::str::from_utf8(cursor.rest()).unwrap()
    };

    assert_eq!(rest("1 + "),      " + ");
    assert_eq!(rest("1 + * 2"),   " + * 2");
    assert_eq!(rest("1 + 2 ) 3"), " ) 3");
    assert_eq!(rest("2 3"),       " 3");
    assert!(<UseAll<Calc>>::parse::<()>("1 + ".into()).is_miss());

    assert!(Calc::parse::<()>("".into()).is_miss());
    assert!(Calc::parse::<()>("+ 1".into()).is_miss());

    assert!(Calc::parse::<()>("(1 + 2".into()).is_fail());
    assert!(Calc::parse::<()>("()".into()).is_fail());
    assert!(Calc::parse::<()>("1 + (2".into()).is_fail());
    assert!(Calc::parse::<()>("1 ? 2".into()).is_fail());
}

#[test]
fn diagnostic() {
    let err = Calc::parse::<Traced>("2 * (1 + 3".into()).expect_fail("should have failed");
    let expect = r#"While matching for: "(" Expr ")"
  at position 4..10
  | 2 * (1 + 3
  |     ^^^^^^

While matching for: "(" Expr ")"
  at position 4..10
  | 2 * (1 + 3
  |     ^^^^^^

While matching for: Expr ")"
  at position 5..10
  | 2 * (1 + 3
  |      ^^^^^

While matching for: ")"
  at position 10..10
  | 2 * (1 + 3
  |           ^
"#;
    println!("{err}");
    assert_eq!(format!("{err}"), expect);
}


/* ====| s-expressions: any number of operands |==== */

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Lisp { Add, Sub, Mul, Div, Max, Min, Lt }

type Call<const HEAD: &'static str, E> =
    destruct!( (_, _, operands, _) = (WsTok<"(">, WsTok<HEAD>, Vec<E>, WsTok<")">) );

operators!{ ('a, E) [Lisp] {
    {
        Out(xs: Call<"+",   E>) => (Self::Add, xs);
        Out(xs: Call<"-",   E>) => (Self::Sub, xs);
        Out(xs: Call<"*",   E>) => (Self::Mul, xs);
        Out(xs: Call<"/",   E>) => (Self::Div, xs);
        Out(xs: Call<"max", E>) => (Self::Max, xs);
        Out(xs: Call<"min", E>) => (Self::Min, xs);
        Out(xs: Call<"<",   E>) => (Self::Lt,  xs);
    }
}}

fn run(tree: &Tree<f64, Lisp>) -> f64 {
    use Lisp::*;
    match tree {
        Tree::Term(n) => *n,
        Tree::List(op, operands) => {
            let v = operands.iter().map(run).collect::<Vec<_>>();
            match op {
                Add => v.iter().sum(),
                Mul => v.iter().product(),
                Sub => match v.split_first() {
                    Some((x, [])) => -x,
                    Some((x, rest)) => x - rest.iter().sum::<f64>(),
                    None => 0.0,
                },
                Div => v.iter().copied().reduce(|a, b| a / b).unwrap_or(1.0),
                Max => v.iter().copied().fold(f64::NEG_INFINITY, f64::max),
                Min => v.iter().copied().fold(f64::INFINITY, f64::min),
                Lt  => num(v.windows(2).all(|w| w[0] < w[1])),
            }
        }
    }
}

table! { s_expressions [Lisp] [run] {
    "42"                 => "42"                  = 42.0;
    "(+ 1 2 3)"          => "(Add 1 2 3)"         =  6.0;
    "(* 2 (+ 1 2) 4)"    => "(Mul 2 (Add 1 2) 4)" = 24.0;
    "(- 5)"              => "(Sub 5)"             = -5.0;
    "(- 10 1 2 3)"       => "(Sub 10 1 2 3)"      =  4.0;
    "(/ 100 2 5)"        => "(Div 100 2 5)"       = 10.0;
    "(max 1 (* 2 3) 4)"  => "(Max 1 (Mul 2 3) 4)" =  6.0;
    "(min 3 (- 1) 2)"    => "(Min 3 (Sub 1) 2)"   = -1.0;
    "(< 1 2 3)"          => "(Lt 1 2 3)"          =  1.0;
    "(< 1 3 2)"          => "(Lt 1 3 2)"          =  0.0;
    "( +  1   (+ 2 3) )" => "(Add 1 (Add 2 3))"   =  6.0;
}}


/* whitespace as an operator */

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Sel { Has, Not, Descendant, Child }

type Pseudo<const NAME: &'static str, E> =
    destruct!( (_, _, _, inner, _) = (Tok<":">, Tok<NAME>, Tok<"(">, E, Tok<")">) );

operators!{ ('a, E) [Sel] {
    {
        Suf(inner: Pseudo<"has", E>) => (Self::Has, inner);
        Suf(inner: Pseudo<"not", E>) => (Self::Not, inner);
    }
    {
        In(_: Ws) => Self::Descendant;
        In(_: WsTok<">">) => Self::Child;
    }
}}

#[test]
fn selectors() {
    let cases = [
        ("div",                      "div"),
        ("a b c",                    "(Descendant a b c)"),
        ("div > p",                  "(Child div p)"),
        ("a b > c d",                "(Descendant (Child (Descendant a b) c) d)"),
        ("div:has(a b) > p",         "(Child (Has div (Descendant a b)) p)"),
        ("li:not(a):has(b > c)",     "(Has (Not li a) (Child b c))"),
    ];

    for (input, expect) in cases {
        let tree = <UseAll<Expr<Ident, Sel, ()>>>::parse::<Traced>(input.into()).must();
        assert_eq!(tree.to_string(), expect, "tree of {input:?}");
    }

    assert!(<UseAll<Expr<Ident, Sel, ()>>>::parse::<()>("div ".into()).is_miss());
}
