//! Expressions from a table of operators. Every operator is a parser, placed before,
//! after, between or around operands; the table says which binds tighter. The result
//! is a `Tree`: a term, or an operator over a list of operands.
//! ```
//! use takion::*;
//!
//! #[derive(Debug, Clone, Copy, PartialEq)]
//! enum Op { Fact, Pow, Neg, Mul, Add, Sub }
//!
//! operators! { [Op] {          // strongest first
//!     { Suf("!") => Self::Fact; }
//!     right { In("^") => Self::Pow; }
//!     { Pre("-") => Self::Neg; }
//!     { In("*") => Self::Mul; }
//!     { In("+") => Self::Add; In("-") => Self::Sub; }
//! }}
//!
//! type Calc = Expr<f64, Op>;
//! let tree = Calc::parse::<()>("-2 ^ 2 * 3! + 1 + 1".into()).must();
//! assert_eq!(tree.to_string(), "(Add (Mul (Neg (Pow 2 2)) (Fact 3)) 1 1)");
//! ```
use super::*;

/// A term, or an operator applied to a list of operands in source order.
/// A chain `a + b + c` is one list: how it folds is up to whoever reads the tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Tree<T, O> {
    Term(T),
    List(O, Vec<Self>),
}

impl<T: Display, O: Debug> Display for Tree<T, O> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Tree::Term(term) => write!(f, "{term}"),
            Tree::List(op, operands) => {
                write!(f, "({op:?}")?;
                operands.iter().try_for_each(|operand| write!(f, " {operand}"))?;
                write!(f, ")")
            }
        }
    }
}

/// What an operator is in the tree: itself, and the operands it parsed on its own.
/// An operator's value converts into it from `Op`, `(Op, operand)` or `(Op, Vec<operand>)`.
pub struct Head<O, X>(pub O, pub Vec<X>);

impl<O, X> From<O> for Head<O, X>              { fn from(op: O) -> Self { Head(op, vec![]) } }
impl<O, X> From<(O, X)> for Head<O, X>         { fn from((op, x): (O, X)) -> Self { Head(op, vec![x]) } }
impl<O, X> From<(O, Vec<X>)> for Head<O, X>    { fn from((op, xs): (O, Vec<X>)) -> Self { Head(op, xs) } }

/// A closed operator is a whole node. Same conversions, and a node is its own.
impl<T, O> From<O> for Tree<T, O>                       { fn from(op: O) -> Self { Tree::List(op, vec![]) } }
impl<T, O> From<(O, Vec<Tree<T, O>>)> for Tree<T, O>   { fn from((op, xs): (O, Vec<Self>)) -> Self { Tree::List(op, xs) } }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Assoc { Left, Right }

/// What can start an operand. Levels count from the strongest, which is 0.
pub enum Operand<O, X> {
    Pre(usize, Head<O, X>),
    Out(X),
}

/// What can follow an operand.
pub enum Operator<O, X> {
    Suf(usize, Head<O, X>),
    In(usize, Assoc, Head<O, X>),
}

/// A table of operators. `operators!` writes this. Both lookups run every operator
/// of their position, at every level, and keep the one that matched the most input;
/// on a tie the one declared first wins. Whether the level suits the place it was
/// found in is for the caller to say: `!=` is not a `!` that was cut short.
/// `E` is the expression being parsed, for operators that contain expressions.
pub trait Operators: Rule + PartialEq + Sized {
    fn operand<'a, T, E, Cx: Ctx>(cursor: Cursor<'a, u8>)
    -> Ret<'a, u8, Operand<Self, Tree<T, Self>>, Cx>
    where E: Parse<'a, u8, Item = Tree<T, Self>>;

    fn operator<'a, T, E, Cx: Ctx>(cursor: Cursor<'a, u8>)
    -> Ret<'a, u8, Operator<Self, Tree<T, Self>>, Cx>
    where E: Parse<'a, u8, Item = Tree<T, Self>>;
}

/// `Expr<Atom, Ops, Gap = SkipWs>`: precedence climbing over `Ops`.
/// - `Gap` is skipped before every operand and operator. `()` makes whitespace matter.
/// - An operator with no operand after it is not part of the expression.
/// - The tightest operators are the first in the table; a level's `right` makes
///   chains of its infix operators nest to the right.
pub struct Expr<Atom, Ops, Gap = SkipWs>(PhantomData<(Atom, Ops, Gap)>);

impl<A, O, G> Rule for Expr<A, O, G> { type This = Txt<"Expr">; }

impl<'a, A, O, G> Parse<'a, u8> for Expr<A, O, G>
where A: Parse<'a, u8>, O: Operators, G: Parse<'a, u8>,
{
    const NULLABLE: bool = A::NULLABLE;
    type Item = Tree<A::Item, O>;

    #[inline]
    fn parse<Cx: Ctx>(cursor: Cursor<'a, u8>) -> Ret<'a, u8, Self::Item, Cx> {
        Self::climb::<Cx>(cursor, usize::MAX)
    }
}

impl<'a, A, O, G> Expr<A, O, G>
where A: Parse<'a, u8>, O: Operators, G: Parse<'a, u8>,
{
    fn climb<Cx: Ctx>(start: Cursor<'a, u8>, below: usize) -> Ret<'a, u8, Tree<A::Item, O>, Cx> {
        let (mut cursor, mut lhs) = Self::operand::<Cx>(skip::<G>(start))??;

        loop {
            let at = skip::<G>(cursor);
            let SubRet::Pass(end, found) = O::operator::<A::Item, Self, Cx>(at)? else { break };

            match found {
                Operator::Suf(level, Head(op, own)) if level < below => {
                    lhs = node(op, Some(lhs), own, None);
                    cursor = end;
                }
                Operator::In(level, assoc, Head(op, own)) if level < below => {
                    let tighter = match assoc { Assoc::Left => level, Assoc::Right => level + 1 };
                    let SubRet::Pass(c, rhs) = Self::climb::<Cx>(end, tighter)? else { break };
                    lhs = infix(assoc, op, lhs, own, rhs);
                    cursor = c;
                }
                // it binds looser than this expression: it is for whoever called
                _ => break,
            }
        }

        Pass(cursor, lhs)
    }

    fn operand<Cx: Ctx>(cursor: Cursor<'a, u8>) -> Ret<'a, u8, Tree<A::Item, O>, Cx> {
        match O::operand::<A::Item, Self, Cx>(cursor)? {
            SubRet::Pass(end, Operand::Out(whole)) => return Pass(end, whole),
            SubRet::Pass(end, Operand::Pre(level, Head(op, own))) => {
                if let SubRet::Pass(c, rhs) = Self::climb::<Cx>(end, level)? {
                    return Pass(c, node(op, None, own, Some(rhs)))
                }
            }
            SubRet::Miss(_) => {}
        }

        let (c, atom) = A::parse::<Cx>(cursor)??;
        Pass(c, Tree::Term(atom))
    }
}

fn skip<'a, G: Parse<'a, u8>>(cursor: Cursor<'a, u8>) -> Cursor<'a, u8> {
    G::parse::<()>(cursor).option().map_or(cursor, |(c, _)| c)
}

fn node<T, O>(op: O, before: Option<Tree<T, O>>, own: Vec<Tree<T, O>>, after: Option<Tree<T, O>>) -> Tree<T, O> {
    Tree::List(op, before.into_iter().chain(own).chain(after).collect())
}

/// `a - b - c` is one list. Only the side the chain grows on is joined, so
/// `a - (b - c)` is two, and an operator that has operands of its own never joins.
fn infix<T, O: PartialEq>(assoc: Assoc, op: O, lhs: Tree<T, O>, own: Vec<Tree<T, O>>, rhs: Tree<T, O>) -> Tree<T, O> {
    match (assoc, own.is_empty(), lhs, rhs) {
        (Assoc::Left, true, Tree::List(o, mut xs), r) if o == op => { xs.push(r); Tree::List(op, xs) }
        (Assoc::Right, true, l, Tree::List(o, mut xs)) if o == op => { xs.insert(0, l); Tree::List(op, xs) }
        (_, _, l, r) => node(op, Some(l), own, Some(r)),
    }
}


/// Declares the operators of `[Op]`, a plain enum, as levels from tightest to loosest.
/// Every line is `Position(parser) => value;` and follows the shape of a `parser!` arm:
/// literals, `name: Type` bindings, or types. Positions:
/// - `Pre`: before an operand, which it takes
/// - `Suf`: after an operand, which it takes
/// - `In`:  between two operands; `right { .. }` before a level makes it right-nesting
/// - `Out`: a whole operand by itself, like `( .. )`
///
/// The value is `Self::Variant`, or `(Self::Variant, operand)` / `(Self::Variant, operands)`
/// when the parser read operands of its own. Those go in the list where they stood in the
/// source. An `Out` value is a tree: `(Self::Variant, operands)`, or an operand as is.
///
/// Literals are `Choose`d, so `"<" | "<="` works in either order. The same literal
/// may be an operator in several positions; the position decides which is looked up.
/// The header `('a, E)` names the lifetime and the expression parser, for operators
/// that contain expressions.
/// ```
/// use takion::*;
///
/// #[derive(Debug, Clone, Copy, PartialEq)]
/// enum Op { Not, Optional, Call }
///
/// operators! { ('a, E) [Op] {
///     { Out((e, _): Commit<WsTok<"(">, (E, WsTok<")">)>) => e; }
///     { Suf((xs, _): Commit<WsTok<"[">, (Intersperse<E, WsTok<",">>, WsTok<"]">)>) => (Self::Call, xs); }
///     { Pre("!") => Self::Not; Suf("?") => Self::Optional; }
/// }}
///
/// type Atoms = Expr<Ident, Op>;
/// let tree = Atoms::parse::<()>("!(a)? [b, c]".into()).must();
/// assert_eq!(tree.to_string(), "(Call (Optional (Not a)) b c)");
/// ```
pub macro operators {
    ([$Op:ty] $levels:tt) => { operators!{ ('a, E) [$Op] $levels } },

    (($lt:lifetime, $E:ident) [$Op:ty]
        { $( $($assoc:ident)? { $( $pos:ident $arg:tt => $Body:expr; )* } )+ }) => {

        impl Rule for $Op { type This = Txt<"Operator">; }

        #[allow(unused_variables, unused_assignments, unused_mut)]
        impl Operators for $Op {
            fn operand<$lt, T, $E, Cx: Ctx>(cursor: Cursor<$lt, u8>)
            -> Ret<$lt, u8, Operand<Self, Tree<T, Self>>, Cx>
            where $E: Parse<$lt, u8, Item = Tree<T, Self>>
            {
                let mut best: Option<(Cursor<$lt, u8>, Operand<Self, Tree<T, Self>>)> = None;
                let mut level = 0usize;
                $( $( operand_arm!([Cx cursor best level] $pos $arg => $Body); )* level += 1; )+

                match best {
                    Some((end, found)) => Pass(end, found),
                    None => Miss(Cx::Info::new::<Self>(cursor, cursor.index)),
                }
            }

            fn operator<$lt, T, $E, Cx: Ctx>(cursor: Cursor<$lt, u8>)
            -> Ret<$lt, u8, Operator<Self, Tree<T, Self>>, Cx>
            where $E: Parse<$lt, u8, Item = Tree<T, Self>>
            {
                let mut best: Option<(Cursor<$lt, u8>, Operator<Self, Tree<T, Self>>)> = None;
                let mut level = 0usize;
                $( {
                    let assoc = assoc!($($assoc)?);
                    $( operator_arm!([Cx cursor best level assoc] $pos $arg => $Body); )*
                } level += 1; )+

                match best {
                    Some((end, found)) => Pass(end, found),
                    None => Miss(Cx::Info::new::<Self>(cursor, cursor.index)),
                }
            }
        }
    },
}

macro assoc {
    ()      => { Assoc::Left  },
    (left)  => { Assoc::Left  },
    (right) => { Assoc::Right },
}

macro operand_arm {
    ([$Cx:ident $cursor:ident $best:ident $level:ident] Pre $arg:tt => $Body:expr) => {
        if let Some((end, head)) = attempt!([$Cx] [$cursor] $arg => $Body)
        && $best.as_ref().is_none_or(|(far, _)| end.index > far.index) {
            $best = Some((end, Operand::Pre($level, Head::from(head))));
        }
    },
    ([$Cx:ident $cursor:ident $best:ident $level:ident] Out $arg:tt => $Body:expr) => {
        if let Some((end, whole)) = attempt!([$Cx] [$cursor] $arg => $Body)
        && $best.as_ref().is_none_or(|(far, _)| end.index > far.index) {
            $best = Some((end, Operand::Out(Tree::from(whole))));
        }
    },
    ([$($ids:ident)*] Suf $arg:tt => $Body:expr) => {},
    ([$($ids:ident)*] In  $arg:tt => $Body:expr) => {},
}

macro operator_arm {
    ([$Cx:ident $cursor:ident $best:ident $level:ident $assoc:ident] Suf $arg:tt => $Body:expr) => {
        if let Some((end, head)) = attempt!([$Cx] [$cursor] $arg => $Body)
        && $best.as_ref().is_none_or(|(far, _)| end.index > far.index) {
            $best = Some((end, Operator::Suf($level, Head::from(head))));
        }
    },
    ([$Cx:ident $cursor:ident $best:ident $level:ident $assoc:ident] In $arg:tt => $Body:expr) => {
        if let Some((end, head)) = attempt!([$Cx] [$cursor] $arg => $Body)
        && $best.as_ref().is_none_or(|(far, _)| end.index > far.index) {
            $best = Some((end, Operator::In($level, $assoc, Head::from(head))));
        }
    },
    ([$($ids:ident)*] Pre $arg:tt => $Body:expr) => {},
    ([$($ids:ident)*] Out $arg:tt => $Body:expr) => {},
}

/// One operator's parser, as `parser!` reads an arm. `Some` with the value it makes, or `None`.
macro attempt {
    ([$Cx:ident] [$cursor:ident] { $($any:tt)* } => $Body:expr) => {
        if let SubRet::Pass(end, _) = <pat!{$($any)*}>::parse::<$Cx>($cursor)? { Some((end, $Body)) } else { None }
    },
    ([$Cx:ident] [$cursor:ident] ( $( $($lit:literal)|+ ),* $(,)? ) => $Body:expr) => {
        if let SubRet::Pass(end, _) = <( $( literal!($($lit)|+), )* )>::parse::<$Cx>($cursor)? { Some((end, $Body)) } else { None }
    },
    ([$Cx:ident] [$cursor:ident] ( $( $id:tt : $ty:ty ),* $(,)? ) => $Body:expr) => {
        if let SubRet::Pass(end, ( $( $id ,)* )) = <( $( $ty ,)* )>::parse::<$Cx>($cursor)? { Some((end, $Body)) } else { None }
    },
    ([$Cx:ident] [$cursor:ident] ( $( $ty:ty ),* $(,)? ) => $Body:expr) => {
        if let SubRet::Pass(end, _) = <( $( $ty ,)* )>::parse::<$Cx>($cursor)? { Some((end, $Body)) } else { None }
    },
}

macro literal {
    ($one:literal)       => { Tok<$one> },
    ($($any:literal)|+)  => { Choose<{ &[ $($any),+ ] }> },
}
