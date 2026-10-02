use std::{any::{Any, TypeId, type_name}, fmt, fmt::Write};
use super::*;


// TODO: decide on a consistent formatting scheme; make a "format" param which decides
// the format in which things are formatted in
/*
Value := {
    | boolean
    | number
    | ('"' string '"')
    | "null"
}

Array := ('[' [Item]<,>* ']')
Object := ('{' [('"' string '"' ':' Item)]<,>* '}')

Item := { Json | Value }
Json := { Object | Array }
*/

/*
Notes about what `takion`s syntax for prints should look like.
Something like a mix between PEG and rusts declarative macros seems
to be what I'm approaching.

`abc`
repeat     :: [a-z]+ == `vec!["a", "b", "c"]`

`a,b,c`
separate  :: ([a-z])<','>+
punctuate :: ([a-z])<','>+ <','>?
repeat    :: [a-z] (',' repeat)?

`a;b;c;`
repeat    :: ([a-z] ',')+          == `vec!["a", ";", "b", ";", "c", ";"]`
terminate :: ([a-z] <';'>)+        == `vec!["a", "b", "c"]`
punctuate :: ([a-z])<';'>+ <';'>?  == `vec!["a", "b", "c"]`
*/

// This probably doesn't need to be a separate trait at all.
pub trait Literal<T: ?Sized + 'static = str> { fn literal() -> &'static T; }

// TODO: Eventual rewrite.
// There needs to be a "grammar shape", the literal parser composition (if applicable)
// A "name"/"ident" if applicable, to enable grammar prints like what's shown below
/*
Value := {
    | boolean
    | number
    | ('"' string '"')
    | "null"
}

Array := ('[' [Item]<,>* ']')
Object := ('{' [('"' string '"' ':' Item)]<,>* '}')

Item := { Json | Value }
Json := { Object | Array }
*/
pub trait Rule {
    type This: FormatType;
    type Mod: FormatType = Txt<"">;
    type Opt: FormatType = Txt<"?">;
    type Fmt: FormatType = (Self::This, Self::Mod);

    /// placeholder for now
    fn type_name() -> &'static str { type_name::<Self>() }

    fn display() -> TypeDisplay<Self::Fmt> where TypeDisplay<Self::Fmt>: Display
        { TypeDisplay(PhantomData) }
    fn stringify() -> String where TypeDisplay<Self::Fmt>: Display
        {  <TypeDisplay<Self::Fmt>>::realize() }
}

pub trait FormatType: Rule {
    fn fmt_type(f: &mut fmt::Formatter<'_>) -> fmt::Result;

    fn realizer() -> impl Display {
        /// This prevents infinite recursions that happen with some primitives rule impls.
        /// ?
        struct Wrapper<T: ?Sized>(std::marker::PhantomData<T>);
        impl<T: ?Sized + FormatType> fmt::Display for Wrapper<T> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { T::fmt_type(f) }
        }
        Wrapper::<Self>(std::marker::PhantomData)
    }

    fn realize() -> String { format!("{}", Self::realizer()) }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct TypeDisplay<T>(pub PhantomData<T>);

impl<T: Rule> FormatType for TypeDisplay<T> {
    fn fmt_type(f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <T as Rule>::Fmt::fmt_type(f)
    }
}

impl<T: Rule> fmt::Display for TypeDisplay<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { <T as Rule>::Fmt::fmt_type(f) }
}

impl<T: Rule> Rule for TypeDisplay<T> where Self: {
    type This = Self;
    type Fmt = Self;
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Txt<const S: &'static str>;

impl<const S: &'static str> Rule for Txt<{S}> { type This = Self; }

impl<const S: &'static str> Literal<str> for Txt<S>
    { fn literal() -> &'static str {S} }

impl<const S: &'static str> FormatType for Txt<S> {
    fn fmt_type(f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_str(S) }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct AsRule<P, R>(pub P, pub PhantomData<R>);

impl<P, R: Rule> Rule for AsRule<P, R> {
    type This = R::This;
    type Opt = R::Opt;
    type Mod = R::Mod;
}

impl<'a, T, P: Parse<'a, T>, R: Rule> Parse<'a, T> for AsRule<P, R> {
    const NULLABLE: bool = P::NULLABLE;
    type Item = P::Item;

    #[inline]
    fn parse<Cx: Ctx>(cursor: Cursor<'a, T>) -> Ret<'a, T, Self::Item, Cx> {
        P::parse::<Cx>(cursor)
            .map_err(|f| Cx::Info::new::<R>(cursor, f.end().unwrap_or(cursor.index)))
    }
}

/// Prints the number as a number
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Num<const N: usize>;

impl<const N: usize> Rule for Num<N> { type This = Self; }
impl<const N: usize> FormatType for Num<N> {
    fn fmt_type(f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{N}") }
}

/// prints values luch as `b'A'` as `A` rather than as the number.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Ascii<const C: u8>;
impl<const C: u8> Rule for Ascii<C> { type This = Self; }
impl<const C: u8> FormatType for Ascii<C> {
    fn fmt_type(f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_char(C as char) }
}
