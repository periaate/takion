use super::*;
use std::marker::ConstParamTy;

/// in `takion` the following semantics:
/// - all repetition is well-founded (the cursor must move each repetetion).
/// - there exists no repetition of "0" matches; always `P+`, for `P*`: Option<Repeat<..>>.
pub struct Repeat<
    Parser,
    const RANGE: Range<usize> = {Range::Full},
>(pub Parser, pub Range<usize>);

impl<P: Rule, const R: Range<usize>> Rule for Repeat<P, {R}> {
    type This = P::This;
    type Mod = Txt<"+">;
    type Opt = Txt<"*">;
    type Fmt = Join<(Self::This, Self::Mod)>;
}

impl<'a, T, P, const R: Range<usize>> Parse<'a, T> for Repeat<P, {R}>
where P: Parse<'a, T> {
    type Item = Vec<P::Item>;
    const NULLABLE: bool = false;

    #[inline(always)]
    fn parse<Cx: Ctx>(cursor: Cursor<'a, T>) -> Ret<'a, T, Self::Item, Cx> {
        if R.upper_bound() == Some(0) {
            return Miss(Cx::Info::new::<Self>(cursor, cursor.index))
        };

        let (mut cur, first) = P::parse::<Cx>(cursor)
            .map_err(|e| e.wrap::<Self>(cursor.index, cursor.index))??;

        let mut matched: usize = 1;
        let mut items = vec![first];

        if cursor.index >= cur.index {
            return Miss(Cx::Info::new::<Self>(cursor, cursor.index))
        };

        while R.do_i_continue(matched)
            && let SubRet::Pass(new, val) = P::parse::<Cx>(cur)?
            && cur.index < new.index
        {
            items.push(val);
            cur = new;
            matched += 1;
        };

        if !R.is_valid(matched) { return Miss(Cx::Info::new::<Self>(cursor, cur.index)) };

        Pass(cur, items)
    }
}

#[derive(ConstParamTy, Copy, Clone, PartialEq, Eq)]
pub enum Range<Unit> {
    AtLeast(Unit      ),
    Between(Unit, Unit),
    AtMost (      Unit),

    Exactly(Unit),
    Full,
}

/// Const compatible rust range pattern syntax for bounding `Repeat`.
/// ```
/// use takion::*;
/// assert!(<UseAll<Repeat<Digit, {count![ ..=2 ]}>>>::parse::<()>("".into()).is_miss());
/// assert!(<UseAll<Repeat<Digit, {count![ ..=2 ]}>>>::parse::<()>("1".into()).is_pass());
/// assert!(<UseAll<Repeat<Digit, {count![ ..=2 ]}>>>::parse::<()>("12".into()).is_pass());
/// assert!(<UseAll<Repeat<Digit, {count![ ..=2 ]}>>>::parse::<()>("123".into()).is_miss());
///
/// assert!(<UseAll<Repeat<Digit, {count![ 2.. ]}>>>::parse::<()>("".into()).is_miss());
/// assert!(<UseAll<Repeat<Digit, {count![ 2.. ]}>>>::parse::<()>("1".into()).is_miss());
/// assert!(<UseAll<Repeat<Digit, {count![ 2.. ]}>>>::parse::<()>("12".into()).is_pass());
/// assert!(<UseAll<Repeat<Digit, {count![ 2.. ]}>>>::parse::<()>("123".into()).is_pass());
///
/// assert!(<UseAll<Repeat<Digit, {count![ 2 ]}>>>::parse::<()>("".into()).is_miss());
/// assert!(<UseAll<Repeat<Digit, {count![ 2 ]}>>>::parse::<()>("1".into()).is_miss());
/// assert!(<UseAll<Repeat<Digit, {count![ 2 ]}>>>::parse::<()>("12".into()).is_pass());
/// assert!(<UseAll<Repeat<Digit, {count![ 2 ]}>>>::parse::<()>("123".into()).is_miss());
///
/// assert!(<UseAll<Repeat<Digit, {count![ ..3 ]}>>>::parse::<()>("".into()).is_miss());
/// assert!(<UseAll<Repeat<Digit, {count![ ..3 ]}>>>::parse::<()>("1".into()).is_pass());
/// assert!(<UseAll<Repeat<Digit, {count![ ..3 ]}>>>::parse::<()>("12".into()).is_pass());
/// assert!(<UseAll<Repeat<Digit, {count![ ..3 ]}>>>::parse::<()>("123".into()).is_miss());
///
/// assert!(<UseAll<Repeat<Digit, {count![ 2..3 ]}>>>::parse::<()>("".into()).is_miss());
/// assert!(<UseAll<Repeat<Digit, {count![ 2..3 ]}>>>::parse::<()>("1".into()).is_miss());
/// assert!(<UseAll<Repeat<Digit, {count![ 2..3 ]}>>>::parse::<()>("12".into()).is_pass());
/// assert!(<UseAll<Repeat<Digit, {count![ 2..3 ]}>>>::parse::<()>("123".into()).is_miss());
///
/// assert!(<UseAll<Repeat<Digit, {count![ 2..=3 ]}>>>::parse::<()>("".into()).is_miss());
/// assert!(<UseAll<Repeat<Digit, {count![ 2..=3 ]}>>>::parse::<()>("1".into()).is_miss());
/// assert!(<UseAll<Repeat<Digit, {count![ 2..=3 ]}>>>::parse::<()>("12".into()).is_pass());
/// assert!(<UseAll<Repeat<Digit, {count![ 2..=3 ]}>>>::parse::<()>("123".into()).is_pass());
/// ```
pub macro count {
    ( $from:literal ..= $upto:literal ) => { Range::Between($from, $upto  ) },
    ( $from:literal ..  $upto:literal ) => { Range::Between($from, $upto-1) },
    (               ..= $upto:literal ) => { Range::AtMost (       $upto  ) },
    (               ..  $upto:literal ) => { Range::AtMost (       $upto-1) },
    ( $from:literal .. )                => { Range::AtLeast($from         ) },

    ( $just:literal ) => { Range::Exactly($just) },
    ( .. ) => { Range::Full },
}

/// ```
/// use takion::*;
///
/// assert_eq!(
///     <Slice<Alt<(Tok<"A">, Tok<"B">)>>>::parse::<()>("ABBABA".into()).must(),
///     "ABBABA".as_bytes(),
/// );
/// ```
pub type Slice<
    P,
    const RANGE: Range<usize> = {Range::Full},
> = Span<Skip<Repeat<Skip<P>, {RANGE}>>>;

/// `Terminate` matches an `S`-terminated list of `P`,
/// e.g. `a;b;c;`
/// ```
/// use takion::*;
///
/// type List = UseAll<Terminate<Letter, Tok<";">>>;
/// assert!(List::parse::<()>("a;b;c;".into()).is_pass());
/// assert!(List::parse::<()>("a;b;c".into()).is_miss());
/// assert!(List::parse::<()>("".into()).is_miss());
/// ```
pub type Terminate<P, S> = Destruct<(Just, Excl), (Intersperse<P, S>, S)>;

/// `Punctuate` matches an `S`-separated, optionally `S`-terminated list of `P`,
/// e.g. `a,b,c` or `a,b,c,`.
/// ```
/// use takion::*;
///
/// type List = UseAll<Punctuate<Letter, Tok<",">>>;
/// assert!(List::parse::<()>("a,b,c".into()).is_pass());
/// assert!(List::parse::<()>("a,b,c,".into()).is_pass());
/// assert!(List::parse::<()>("".into()).is_miss());
/// ```
pub type Punctuate<P, S> = Destruct<(Just, Excl), (Intersperse<P, S>, Option<S>)>;

/// `Intersperse` matches an `S`-separated, but not `S`-terminated, list of `P`,
/// e.g. `a,b,c` but not `a,b,c,`.
/// ```
/// use takion::*;
///
/// type List = UseAll<Intersperse<Letter, Tok<",">>>;
/// assert!(List::parse::<()>("a,b,c".into()).is_pass());
/// assert!(List::parse::<()>("a,b,c,".into()).is_miss());
/// assert!(List::parse::<()>("".into()).is_miss());
/// ```
///
/// TODO: add range const/parameter.
pub struct Intersperse<P, S>(pub P, pub S);

pub type Separate<P, S> = Intersperse<P, S>;

impl<P: Rule, S: Rule> Rule for Intersperse<P, S> {
    type This = Join<(Txt<"$(">, P, Txt<")">, S)>;
    type Mod = Txt<"+">;
    type Opt = Txt<"*">;
}

impl<'a, T, P: Parse<'a, T>, S: Parse<'a, T>> Parse<'a, T> for Intersperse<P, S> {
    type Item = Vec<P::Item>;

    #[inline(always)]
    fn parse<Cx: Ctx>(cursor: Cursor<'a, T>) -> Ret<'a, T, Self::Item, Cx> {
        let (mut cur, first) = P::parse::<Cx>(cursor)
            .map_err(|e| e.wrap::<Self>(cursor.index, cursor.index))??;

        if cursor.index >= cur.index { return Miss(Cx::Info::new::<Self>(cursor, cursor.index)) };

        let mut items = vec![first];

        while let SubRet::Pass(new, (_, val)) = <(S, P)>::parse::<Cx>(cur)?
        && cur.index < new.index {
            items.push(val);
            cur = new;
        };
        Pass(cur, items)
    }
}

impl Range<usize> {
    pub const fn lower_bound(self) -> usize {
        match self {
            Range::AtLeast(n) => n,
            Range::Between(n, _) => n,
            Range::AtMost(_) => 1,
            Range::Exactly(n) => n,
            Range::Full => 1,
        }
    }

    pub const fn upper_bound(self) -> Option<usize> {
        match self {
            Range::AtLeast(_) => None,
            Range::Between(_, m) => Some(m),
            Range::AtMost(m) => Some(m),
            Range::Exactly(n) => Some(n),
            Range::Full => None,
        }
    }
}


impl<Unit> Range<Unit> {
    #[inline(always)]
    pub fn do_i_continue(self, unit: Unit) -> bool where Unit: Ord + Eq {
        match self {
            Range::Between(from, upto) if unit >= upto    => { false }
            Range::AtMost (      upto) if unit >= upto    => { false }
            Range::Exactly(exactly   ) if unit >= exactly => { false }

            _ => { true }
        }
    }

    #[inline(always)]
    pub fn is_valid(self, unit: Unit) -> bool
    where Unit: Ord + Eq {
        match self {
            Range::AtLeast(lo) =>     lo<=unit,
            Range::Between(lo, hi) => lo<=unit && unit<=hi,
            Range::AtMost(hi) =>                  unit<=hi,
            Range::Exactly(eq) =>     unit==eq,
            Range::Full => true,
        }
    }
}



/// `Vec<P>` finds one or more `P` -- `P+`. For zero-or-more `P` (`P*`): `Option<Vec<P>>`.
/// ```
/// use takion::*;
///
/// type Letters = Vec<Letter>;
/// assert!(Letters::parse::<()>("abc".into()).is_pass());
/// assert!(Letters::parse::<()>("".into()).is_miss());
/// ```
impl<'a, T, K: Parse<'a, T>> Parse<'a, T> for Vec<K> {
    const NULLABLE: bool = false;
    type Item = Vec<K::Item>;
    #[inline(always)]
    fn parse<Cx: Ctx>(cursor: Cursor<'a, T>) -> Ret<'a, T, Self::Item, Cx> {
        <Repeat<K, {Range::Full}>>::parse(cursor)
    }
}

impl<T: Rule> Rule for Vec<T> {
    type This = T::This;
    type Mod = Txt<"+">;
    type Opt = Txt<"*">;
    type Fmt = Join<(Self::This, Self::Mod)>;
}


#[test]
fn rep_new() {
    let inter = "a, b, c, d, e";
    let term = "a, b, c, d, e,";

    type Inter = UseAll<Intersperse<Letter, (Tok<",">, Option<Ws>)>>;
    type Term = UseAll<Terminate<Letter, (Tok<",">, Option<Ws>)>>;
    type Punct = UseAll<Punctuate<Letter, (Tok<",">, Option<Ws>)>>;

    assert!(Inter::parse::<()>(inter.into()).is_pass());
    assert!(Term::parse::<()>(term.into()).is_pass());
    assert!(Punct::parse::<()>(inter.into()).is_pass());
    assert!(Punct::parse::<()>(term.into()).is_pass());

    assert!(Inter::parse::<()>(term.into()).is_miss());
    assert!(Term::parse::<()>(inter.into()).is_miss());
}

#[test]
fn repetitions() {
    let input = ". ..";
    assert!(<Vec<Tok<" ">>>::parse::<Normal>(input.into()).is_miss());
    assert!(<(Vec<Tok<" ">>, End)>::parse::<Normal>(input.into()).is_miss());
    assert!(<UseAll<Vec<Tok<" ">>>>::parse::<Normal>(input.into()).is_miss());
    assert!(<UseAll<(Vec<Tok<" ">>, End)>>::parse::<Normal>(input.into()).is_miss());
    assert!(<UseAll<Vec<pat!["."|" "]>>>::parse::<Normal>(input.into()).is_pass());
    assert!(<(UseAll<Vec<pat!["."|" "]>>, End)>::parse::<Normal>(input.into()).is_pass());

    struct Null;
    impl Rule for Null { type This = Txt<"Null">; }
    impl<'a, T> Parse<'a, T> for Null {
        const NULLABLE: bool = false;
        type Item = Null;

        #[inline]
        fn parse<Cx: Ctx>(cursor: Cursor<'a, T>) -> Ret<'a, T, Self::Item, Cx> {
            Pass(cursor, Null)
        }
    }

    // doesn't infinite loop
    assert!(<Vec<Null>>::parse::<Normal>(input.into()).is_miss());
}

/// `Until<C, D = Advance<1>>` repeatedly applies `D` while `C` doesn't match, then stops
/// (without consuming what `C` matched) and returns everything skipped as a span.
/// ```
/// use takion::*;
///
/// /// Same as `Until<Tok<";">, Advance<1>>`, `Advance<1>` is default for `Until`.
/// type UpTo = Until<Tok<";">>;
/// let (cursor, skipped) = UpTo::parse::<()>("hello;world".into()).unwrap().unwrap();
/// assert_eq!(std::str::from_utf8(skipped).unwrap(), "hello");
/// assert_eq!(std::str::from_utf8(cursor.rest()).unwrap(), ";world");
/// ```
#[derive(Debug)]
pub struct Until<Condition, DoThis = Advance<1>>(PhantomData<(Condition, DoThis)>);

impl<'a, T: 'a, C: Parse<'a, T>, D: Parse<'a, T>> Parse<'a, T> for Until<C, D> {
    const NULLABLE: bool = true;
    type Item = &'a [T];

    #[inline]
    fn parse<Cx: Ctx>(start_cursor: Cursor<'a, T>) -> Ret<'a, T, Self::Item, Cx> {
        let mut cursor = start_cursor;

        loop {
            if let SubRet::Pass(_, _) = C::parse::<Cx>(cursor)
                .map_err(|err| err.wrap::<Self>(start_cursor.index, cursor.index))? {
                return Pass(cursor, start_cursor.slice_between(&cursor));
            };

            let (next_cursor, _) = D::parse::<Cx>(cursor)
                .map_miss(|err| err.wrap::<Self>(start_cursor.index, cursor.index))??;

            if cursor.index >= next_cursor.index {
                return Miss(Cx::Info::new::<Self>(cursor, cursor.index))
            };

            cursor = next_cursor;
        }
    }
}

impl<Cond: Rule, DoThis: Rule> Rule for Until<Cond, DoThis> {
    type This = (
        Txt<"$(">,
        Option<Vec<DoThis::Fmt>>,
        Join<(Txt<"?:">, Cond::Fmt)>,
        Txt<")">,
    );
    type Mod = Txt<"">; type Opt = Txt<"*">;
}

