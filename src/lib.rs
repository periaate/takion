//! # takion
//! Type-level parser system in next-generation rust.
//! The root has what writing and using parsers needs. Everything else stays in its module.
//! TODO: a prelude, eventually.

#![expect(incomplete_features, reason = "str in const position is experimental/unstable")]
#![feature(adt_const_params, unsized_const_params)]
#![feature(associated_type_defaults)]
#![feature(try_trait_v2, try_trait_v2_residual)]

#![expect(unused_parens, reason = "The way we use declarative macros needs this")]
#![feature(decl_macro)]

// // the current error management is java tiers of 0 signal pure noise boilerplate.
// // this was an attempt to make it just a bit better. I don't know what the actual
// // right way is.
// pub macro wrap($cursor:ident, $($rest:tt)+) {
//     |e| e.wrap::<$($rest)+>($cursor.index, e.end().unwrap_or($cursor.index))
// }

pub(crate) use std::fmt::{self, Debug, Display};
use std::marker::{ConstParamTy, PhantomData};

pub(crate) use crate as takion;
pub(crate) use Cursor as Cur;

pub mod testing;

mod cursor;
mod ret;
mod macros;
pub mod error;
pub mod formatting;

/* Combinators */
pub mod movement;
pub mod repetitions;
pub mod alternation;
pub mod sequence;
pub mod destructuring;
pub mod recovery;
pub mod capture;
pub mod pipe;
pub mod expr;

/* Strings */
pub mod strings;
pub mod syntax;
pub mod primitives;


// Inside the crate everything is at the root, for `use crate::*`.
pub(crate) use {
    cursor::*, ret::*, error::*, formatting::*,
    movement::*, repetitions::*, alternation::*, sequence::*,
    destructuring::*, recovery::*, capture::*, pipe::*,
    strings::*, syntax::*,
    macros::impl_tuple::tuples,
};

pub use {
    cursor::Cursor,
    ret::{Ret, SubRet},
    error::{Ctx, Info, Normal, Traced},
    formatting::{Rule, Txt, AsRule},

    alternation::{Alt, Or},
    repetitions::{Repeat, Range, count, Slice, Intersperse, Terminate, Punctuate, Until},
    destructuring::{Destruct, Incl, Excl, Just},
    recovery::{Cut, Commit},
    capture::{Span, Skip},
    movement::{Advance, Next, Rest, Null, End, UseAll, Not, Unit},
    pipe::Pipe,

    strings::{Tok, Between, Digit, Letter, Hex, Case, WhiteSpace, Ws, SkipWs, WsTok},
    strings::{UTF8, Coax, ToCow, Stringify, Unescape, unescape, Map},
    syntax::{Ident, QuotedStr, QuotedString, ranges, Ranges},

    macros::{pattern::{self, pat, seq}, parser::parser, parser::parse, destruct::destruct},
};
pub use Ret::*;


/// A parser as a type. Its structure is the grammar.
/// ```
/// use takion::*;
///
/// type Greeting = (Tok<"Hello, ">, Ident);
/// assert_eq!(Greeting::parse::<()>("Hello, World".into()).must(), ("Hello, ", "World"));
/// ```
pub trait Parse<'a, T>: Sized + Rule {
    const NULLABLE: bool = false;

    type Item;
    fn parse<Cx: Ctx>(cursor: Cursor<'a, T>) -> Ret<'a, T, Self::Item, Cx>;
}

/// A parser as a value, for when it has state or has to be passed around.
/// `TypeParser` lowers any `Parse` into one.
pub trait Parser<'a, T> {
    type Item;
    fn parse<Cx: Ctx>(&mut self, cursor: Cursor<'a, T>) -> Ret<'a, T, Self::Item, Cx>;
}

/// Lowers the type level `P` into a `Parser`.
/// ```
/// use takion::*;
///
/// let mut digit = TypeParser::<Digit>::new();
/// assert!(digit.parse::<()>("7".into()).is_pass());
/// ```
pub struct TypeParser<P>(PhantomData<P>);

impl<P> TypeParser<P> {
    pub const fn new() -> Self { Self(PhantomData) }
}

impl<'a, T, P: Parse<'a, T>> Parser<'a, T> for TypeParser<P> {
    type Item = P::Item;

    #[inline(always)]
    fn parse<Cx: Ctx>(&mut self, cursor: Cursor<'a, T>) -> Ret<'a, T, Self::Item, Cx> {
        P::parse::<Cx>(cursor)
    }
}

impl<'a, T> Parser<'a, T> for () {
    type Item = ();
    fn parse<Cx: Ctx>(&mut self, cursor: Cursor<'a, T>) -> Ret<'a, T, Self::Item, Cx> {
        Pass(cursor, ())
    }
}

impl Rule for () { type This = Txt<"">; }

impl<'a, T> Parse<'a, T> for () {
    const NULLABLE: bool = true;
    type Item = ();
    fn parse<Cx: Ctx>(cursor: Cursor<'a, T>) -> Ret<'a, T, Self::Item, Cx> {Pass(cursor, ())}
}



#[test]
fn use_all() {
    let input = "Hello, World!";
    assert!(<UseAll<Tok<"Hello,">>>::parse::<Normal>(input.into()).is_miss());

    type Parser = UseAll<(Tok<"Hello,">, Vec<Alt<(Letter, Ws, Tok<",">, Tok<"!">)>>)>;
    assert!(<Parser>::parse::<Normal>(input.into()).is_pass());
}
