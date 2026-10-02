use crate::*;

/// Advances the cursor by `By` units, whatever they are.
/// ```
/// use takion::*;
///
/// assert_eq!(<Advance<2>>::parse::<()>("abc".into()).must(), b"ab");
/// assert!(<Advance<4>>::parse::<()>("abc".into()).is_miss());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Advance<const By: usize = 1>;

/// One unit.
pub type Next = Advance<1>;
/// The rest of the input.
pub type Rest = Advance<{ usize::MAX }>;
/// Nothing. Always matches.
pub type Null = Advance<0>;

impl<'a, T: 'a, const By: usize> Parse<'a, T> for Advance<By> {
    const NULLABLE: bool = By == 0;
    type Item = &'a [T];

    #[inline]
    fn parse<Cx: Ctx>(cursor: Cursor<'a, T>) -> Ret<'a, T, Self::Item, Cx> {
        if By == 0 {  return Pass(cursor, &[]) }
        if By == usize::MAX { return Pass(cursor.advance_upto(usize::MAX), cursor.rest()) }
        if let Some((c, v)) = cursor.take(By)
             { Pass(c, v) }
        else { Miss(Cx::Info::new::<Self>(cursor,cursor.index)) }
    }
}

impl<const By: usize> Rule for Advance<By> {
    type This = Txt<"_">;
    type Mod = Txt<"">;
    type Opt = Txt<"*">;
}

/// Matches any single unit and returns it.
/// ```
/// use takion::*;
///
/// assert_eq!(Unit::parse::<()>("a".into()).must(), b'a');
/// assert!(Unit::parse::<()>("".into()).is_miss());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Unit;

impl Rule for Unit { type This = Txt<"_">; }

impl<'a, T: 'a + Copy> Parse<'a, T> for Unit {
    type Item = T;

    #[inline]
    fn parse<Cx: Ctx>(cursor: Cursor<'a, T>) -> Ret<'a, T, Self::Item, Cx> {
        let Some((new, &val)) = cursor.take_one() else {
            return Miss(Cx::Info::new::<Self>(cursor, cursor.index))
        };
        Pass(new, val)
    }
}

/// Matches only where `P` doesn't, consuming nothing.
/// ```
/// use takion::*;
///
/// type Consonant = (Not<pat!{"a"|"e"|"i"|"o"|"u"}>, Between<b'a', b'z'>);
/// assert!(Consonant::parse::<()>("b".into()).is_pass());
/// assert!(Consonant::parse::<()>("a".into()).is_miss());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Not<P>(pub P);

impl<P: Rule> Rule for Not<P> { type This = Join<(Txt<"!">, P::Fmt)>; }

impl<'a, T, P: Parse<'a, T>> Parse<'a, T> for Not<P> {
    const NULLABLE: bool = true;
    type Item = ();

    #[inline]
    fn parse<Cx: Ctx>(cursor: Cursor<'a, T>) -> Ret<'a, T, Self::Item, Cx> {
        match P::parse::<Cx>(cursor)? {
            SubRet::Pass(..) => Miss(Cx::Info::new::<Self>(cursor, cursor.index)),
            SubRet::Miss(_) => Pass(cursor, ()),
        }
    }
}

/// Matches `P` only if it consumes the rest of the input.
/// ```
/// use takion::*;
///
/// assert!(<UseAll<Tok<"ab">>>::parse::<()>("ab".into()).is_pass());
/// assert!(<UseAll<Tok<"a">>>::parse::<()>("ab".into()).is_miss());
/// ```
pub struct UseAll<P>(PhantomData<P>);
impl<P: Rule> Rule for UseAll<P> { type This = P::This; }

impl<'a, T, P: Parse<'a, T>> Parse<'a, T> for UseAll<P> {
    const NULLABLE: bool = P::NULLABLE;
    type Item = P::Item;

    #[inline(always)]
    fn parse<Cx: Ctx>(cursor: Cursor<'a, T>) -> Ret<'a, T, Self::Item, Cx> {
        let (current, output) = <P>::parse::<Cx>(cursor)??;
        if !current.is_empty() {
            return Miss(Cx::Info::new::<Self>(cursor, current.index))
        }
        Ret::Pass(current, output)
    }
}

/// The end of the input.
/// ```
/// use takion::*;
///
/// assert!(<(Tok<"a">, End)>::parse::<()>("a".into()).is_pass());
/// assert!(<(Tok<"a">, End)>::parse::<()>("ab".into()).is_miss());
/// ```
pub type End = Skip<UseAll<Advance<0>>>;
