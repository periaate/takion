use super::*;

pub trait Precedence {
    fn prec(self) -> u8;
    fn assoc(self) -> Assoc;
}

pub enum Assoc { Left, Right }

impl<A, B> Rule for Expr<A, B> { type This = Txt<"Expr">; }

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr<Term, Oper> {
    Expr(Box<(Self, Oper, Self)>),
    Term(Term),
}

impl<'a, Term: Parse<'a, u8>, Op: Parse<'a, u8>> Parse<'a, u8> for Expr<Term, Op>
    where Op::Item: Precedence + Copy
{
    const NULLABLE: bool = Term::NULLABLE;
    type Item = Expr<Term::Item, Op::Item>;
    #[inline]
    fn parse<Cx: Ctx>(cursor: Cursor<'a, u8>) -> Ret<'a, u8, Self::Item, Cx>
    { Self::parse_prec::<Cx>(cursor, 0) }
}

impl<'a, Term: Parse<'a, u8>, Op: Parse<'a, u8>> Expr<Term, Op>
    where Op::Item: Precedence + Copy,
{
    #[inline]
    fn parse_prec<Cx: Ctx>(start: Cursor<'a, u8>, min_prec: u8)
    -> Ret<'a, u8, Expr<Term::Item, Op::Item>, Cx> {
        let (mut cursor, first) = Term::parse::<Cx>(start)??;

        let mut lhs = Expr::Term(first);

        loop {
            use SubRet::Pass;
            let Pass(c, op) = Op::parse::<Cx>(cursor)? else { break };
            if op.prec() < min_prec { break }
            let next_min_prec = match op.assoc() {
                Assoc::Left => op.prec() + 1,
                Assoc::Right => op.prec(),
            };

            let Pass(c, rhs) = Self::parse_prec::<Cx>(c, next_min_prec)? else {
                return Miss(Cx::Info::new::<Self>(c, start.index))
            };

            cursor = c;
            lhs = Expr::Expr(Box::new((lhs, op, rhs)));
        }

        Pass(cursor, lhs)
    }
}
