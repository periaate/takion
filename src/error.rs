use std::{collections::HashSet, ops::Not};

use super::*;


// makes using Ctx as a type param without `PhantomData` possible where-ever
// you also need a `Cursor`.
pub trait Context<'a, T: 'a>: Sized + Ctx {
    type Cursor = crate::cursor::Cursor<'a, T>;
    type Info: Info<'a, T>;
}

impl<'a, T: 'a, P: Ctx> Context<'a, T> for P {
    type Cursor = crate::cursor::Cursor<'a, T>;
    type Info = P::Info<'a, T>;
}

// uhhh I'm not 100% certain if this indirection step needs to exist. Probably doesn't.
// Though it does make the syntax cleaner/easier at call site.
pub trait Ctx: Sized {
    type Info<'a, T: 'a>: Info<'a, T>;
}

pub trait Info<'a, T>: Sized {
    fn new<P: Rule>(from: Cur<'a, T>, upto: usize) -> Self;
    fn wrap<P: Rule>(self, from: usize, upto: usize) -> Self;

    fn end(&self) -> Option<usize> { None }
    fn span(&self) -> Option<(usize, usize)> { None }
}

pub struct Normal;
impl Ctx for Normal { type Info<'a, T: 'a> = Nothing; }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Nothing;

impl Ctx for () { type Info<'a, T: 'a> = Nothing; }
impl<'a, T> Info<'a, T> for Nothing {
    #[inline(always)] fn new<P: Rule>(_: Cur<'a, T>, _: usize) -> Self { Nothing }
    #[inline(always)] fn wrap<P: Rule>(mut self, _: usize, _: usize) -> Self { Nothing }
}

/// ```compile_fail
/// use takion::error::*;
/// <Traced<0>>::new();
/// ```
pub struct Traced<const DEPTH: usize = 5>;

impl<const DEPTH: usize> Ctx for Traced<DEPTH> {
    type Info<'a, T: 'a> = Frame<'a, T, DEPTH>;
}

#[derive(Clone, Copy)]
pub struct Frame<'a, T, const DEPTH: usize = 5> {
    source: &'a [T],
    depth: usize,
    nest: [Option<Target>; DEPTH]
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct Target((usize, usize), fn(&mut fmt::Formatter<'_>) -> fmt::Result);


impl ::std::fmt::Display for Nothing {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) ->  ::std::fmt::Result {
        f.write_str("Nothing")
    }
}

impl<'a, T, const D: usize> Frame<'a, T, {D}> {
    fn full(&self) -> bool { D <= self.depth }
    fn push(&mut self, input: Target) {
        if self.full() { return; }
        self.nest[self.depth] = Some(input);
        self.depth += 1;
    }
}


impl<'a, T, const DEPTH: usize> Info<'a, T> for Frame<'a, T, {DEPTH}> {
    fn wrap<P: Rule>(mut self, from: usize, upto: usize) -> Self {
        if self.full() { return self; }
        self.push(Target((
            from,
            self.nest[self.depth.saturating_sub(1)].map(|Target((_, to), ..)| to.max(upto)).unwrap_or(upto),
        ), P::Fmt::fmt_type));
        self
    }

    fn end(&self) -> Option<usize> { self.nest[self.depth-1].map(|Target((_, end), ..)| end) }
    fn span(&self) -> Option<(usize, usize)> {
        let (from, upto) = self.nest[0].map(|Target((from, upto), ..)| (from, upto))?;
        if self.depth <= 1 { return Some((from, upto)) };
        Some((
            self.nest[0].map(|Target((from, _), ..)| from)?,
            self.nest[self.depth-1].map(|Target((_, upto), ..)| upto)?,
        ))
    }

    fn new<P: Rule>(from: Cur<'a, T>, upto: usize) -> Self {
        const { if DEPTH == 0 { panic!("Traced must have a Depth >= 1, is 0!"); } };

        let mut arr = [None; DEPTH];
        arr[0] = Some(Target((from.index, upto), P::Fmt::fmt_type));
        Frame {
            source: from.source,
            depth: 1,
            nest: arr,
        }
    }
}

// scuffed, ok for now; TODO: rewrite/rethink all of the inductive stuff
impl<'a, const DEPTH: usize> std::fmt::Display for Frame<'a, u8, {DEPTH}> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let src = String::from_utf8_lossy(self.source);
        let mut set = HashSet::new();

        let mut first = true;
        self.nest.into_iter()
            .take_while(Option::is_some)
            .flatten()
            .filter(|v| set.insert(*v))
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .map(|Target(span, rule)| {
                if !first { writeln!(f, ""); } else { first = false; };
                write!(f, "While matching for: ")?;
                (rule)(f)?;
                writeln!(f)?;
                writeln!(f, "  at position {}..{}", span.0, span.1)?;
                writeln!(f, "  | {src}")?;
                writeln!(f, "  | {}{}", " ".repeat(span.0), "^".repeat((span.1 - span.0).max(1)))?;

                Ok(())
            })
                .collect::<Result<Vec<_>, _>>()?;
        Ok(())
    }
}

impl<'a, const DEPTH: usize> std::fmt::Debug for Frame<'a, u8, {DEPTH}> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}

