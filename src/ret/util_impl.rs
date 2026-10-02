// TODO: this file alone is like 15% of takion, but is like 0.5% of the content, NUKE!!!!!!!
use std::ops::{ControlFlow, FromResidual, Residual, Try};
use super::*;

impl<'a, Unit, Match: Debug, Cx: Ctx> Debug for Ret<'a, Unit, Match, Cx>
where Cx::Info<'a, Unit>: Debug {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Ret::Pass(c, m) => f.debug_tuple("Pass").field(c).field(m).finish(),
            Ret::Miss(i) => f.debug_tuple("Miss").field(i).finish(),
            Ret::Fail(i) => f.debug_tuple("Fail").field(i).finish(),
        }
    }
}

impl<'a, Unit, Match: Clone, Cx: Ctx> Clone for Ret<'a, Unit, Match, Cx>
where Cx::Info<'a, Unit>: Clone {
    fn clone(&self) -> Self {
        match self {
            Ret::Pass(c, m) => Ret::Pass(*c, m.clone()),
            Ret::Miss(i) => Ret::Miss(i.clone()),
            Ret::Fail(i) => Ret::Fail(i.clone()),
        }
    }
}

impl<'a, Unit, Match: Copy, Cx: Ctx> Copy for Ret<'a, Unit, Match, Cx>
where Cx::Info<'a, Unit>: Copy {}

impl<'a, Unit, Match: PartialEq, Cx: Ctx> PartialEq for Ret<'a, Unit, Match, Cx>
where Cx::Info<'a, Unit>: PartialEq, Unit: PartialEq {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Ret::Pass(c1, m1), Ret::Pass(c2, m2)) => c1 == c2 && m1 == m2,
            (Ret::Miss(a), Ret::Miss(b)) | (Ret::Fail(a), Ret::Fail(b)) => a == b,
            _ => false,
        }
    }
}

impl<'a, Unit, Match: Eq, Cx: Ctx> Eq for Ret<'a, Unit, Match, Cx>
where Cx::Info<'a, Unit>: Eq, Unit: Eq {}

impl<'a, Unit, Match: Debug, Cx: Ctx> Debug for SubRet<'a, Unit, Match, Cx>
where Cx::Info<'a, Unit>: Debug {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SubRet::Pass(c, m) => f.debug_tuple("Pass").field(c).field(m).finish(),
            SubRet::Miss(i) => f.debug_tuple("Miss").field(i).finish(),
        }
    }
}

impl<'a, Unit, Match: Clone, Cx: Ctx> Clone for SubRet<'a, Unit, Match, Cx>
where Cx::Info<'a, Unit>: Clone {
    fn clone(&self) -> Self {
        match self {
            SubRet::Pass(c, m) => SubRet::Pass(*c, m.clone()),
            SubRet::Miss(i) => SubRet::Miss(i.clone()),
        }
    }
}

impl<'a, Unit, Match: Copy, Cx: Ctx> Copy for SubRet<'a, Unit, Match, Cx>
where Cx::Info<'a, Unit>: Copy {}

impl<'a, Unit, Match: PartialEq, Cx: Ctx> PartialEq for SubRet<'a, Unit, Match, Cx>
where Cx::Info<'a, Unit>: PartialEq, Unit: PartialEq {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (SubRet::Pass(c1, m1), SubRet::Pass(c2, m2)) => c1 == c2 && m1 == m2,
            (SubRet::Miss(a), SubRet::Miss(b)) => a == b,
            _ => false,
        }
    }
}

impl<'a, Unit, Match: Eq, Cx: Ctx> Eq for SubRet<'a, Unit, Match, Cx>
where Cx::Info<'a, Unit>: Eq, Unit: Eq {}


impl<'a, Unit, Match, Cx: Ctx> Ret<'a, Unit, Match, Cx> {
    pub const fn is_pass(&self) -> bool { matches!(self, Ret::Pass(..)) }
    pub const fn is_miss(&self) -> bool { matches!(self, Ret::Miss(..)) }
    pub const fn is_fail(&self) -> bool { matches!(self, Ret::Fail(..)) }

    pub fn result_option(self) -> Result<Option<(Cursor<'a, Unit>, Match)>, Cx::Info<'a, Unit>>
    where Cx::Info<'a, Unit>: Display {
        match self {
            Ret::Pass(cursor, value) => Ok(Some((cursor, value))),
            Ret::Miss(info) => Ok(None),
            Ret::Fail(info) => Err(info),
        }
    }

    #[track_caller]
    pub fn must(self) -> Match where Cx::Info<'a, Unit>: Display {
        self.unwrap().unwrap().1
    }

    #[track_caller]
    pub fn unwrap(self) -> SubRet<'a, Unit, Match, Cx> where Cx::Info<'a, Unit>: Display {
        match self {
            Ret::Pass(cursor, value) => SubRet::Pass(cursor, value),
            Ret::Miss(info) => SubRet::Miss(info),
            Ret::Fail(info) => panic!("called `Ret::unwrap` on a `Fail`:\n\n{info}"),
        }
    }

    #[track_caller]
    pub fn expect(self, msg: &str) -> SubRet<'a, Unit, Match, Cx> where Cx::Info<'a, Unit>: Display {
        match self {
            Ret::Pass(cursor, value) => SubRet::Pass(cursor, value),
            Ret::Miss(info) => SubRet::Miss(info),
            Ret::Fail(info) => panic!("{msg}\n\n{info}"),
        }
    }

    pub fn some(self) -> Option<Match> {
        match self { Ret::Pass(_, value) => Some(value), _ => None }
    }

    pub fn ok(self) -> Result<Match, String> where Match: Debug, Cx::Info<'a, Unit>: Debug {
        match self {
            Ret::Pass(_, value) => Ok(value),
            Ret::Miss(info) | Ret::Fail(info) => Err(format!("{:?}", info)),
        }
    }

    pub fn option(self) -> Option<(Cursor<'a, Unit>, Match)> {
        match self { Ret::Pass(cursor, value) => Some((cursor, value)), _ => None }
    }

    pub fn result(self) -> Result<(Cursor<'a, Unit>, Match), Cx::Info<'a, Unit>> {
        match self {
            Ret::Pass(cursor, value) => Ok((cursor, value)),
            Ret::Miss(info) | Ret::Fail(info) => Err(info),
        }
    }

    pub fn unwrap_err(self) -> Cx::Info<'a, Unit> where Match: Debug, Cx::Info<'a, Unit>: Debug {
        match self {
            Ret::Pass(..) => panic!("unwrap_err on a Pass: {:#?}", self),
            Ret::Miss(info) | Ret::Fail(info) => info,
        }
    }

    pub fn unwrap_miss(self) -> Cx::Info<'a, Unit> where Match: Debug, Cx::Info<'a, Unit>: Debug {
        match self {
            Ret::Miss(info) => info,
            _ => panic!("unwrap_miss on a {:#?}", self),
        }
    }

    pub fn unwrap_fail(self) -> Cx::Info<'a, Unit> where Match: Debug, Cx::Info<'a, Unit>: Debug {
        match self {
            Ret::Fail(info) => info,
            _ => panic!("unwrap_fail on a {:#?}", self),
        }
    }

    pub fn expect_err(self, msg: &str) -> Cx::Info<'a, Unit> where Match: Debug, Cx::Info<'a, Unit>: Debug {
        match self {
            Ret::Pass(..) => panic!("{}: {:#?}", msg, self),
            Ret::Miss(info) | Ret::Fail(info) => info,
        }
    }

    pub fn expect_miss(self, msg: &str) -> Cx::Info<'a, Unit> where Match: Debug, Cx::Info<'a, Unit>: Debug {
        match self {
            Ret::Miss(info) => info,
            _ => panic!("{}: {:#?}", msg, self),
        }
    }

    pub fn expect_fail(self, msg: &str) -> Cx::Info<'a, Unit> where Match: Debug, Cx::Info<'a, Unit>: Debug {
        match self {
            Ret::Fail(info) => info,
            _ => panic!("{}: {:#?}", msg, self),
        }
    }

    #[inline(always)]
    pub fn map<F, N>(self, f: F) -> Ret<'a, Unit, N, Cx> where F: FnOnce(Match) -> N {
        match self {
            Ret::Pass(c, v) => Ret::Pass(c, f(v)),
            Ret::Miss(info) => Ret::Miss(info),
            Ret::Fail(info) => Ret::Fail(info),
        }
    }

    #[inline(always)]
    pub fn map_err<F>(self, f: F) -> Self where F: FnOnce(Cx::Info<'a, Unit>) -> Cx::Info<'a, Unit> {
        match self {
            Ret::Miss(info) => Ret::Miss(f(info)),
            Ret::Fail(info) => Ret::Fail(f(info)),
            other => other,
        }
    }

    #[inline(always)]
    pub fn map_miss<F>(self, f: F) -> Self where F: FnOnce(Cx::Info<'a, Unit>) -> Cx::Info<'a, Unit> {
        match self {
            Ret::Miss(info) => Ret::Miss(f(info)),
            other => other,
        }
    }

    #[inline(always)]
    pub fn map_fail<F>(self, f: F) -> Self where F: FnOnce(Cx::Info<'a, Unit>) -> Cx::Info<'a, Unit> {
        match self {
            Ret::Fail(info) => Ret::Fail(f(info)),
            other => other,
        }
    }

    #[inline(always)]
    pub fn inspect<F>(self, f: F) -> Self where F: FnOnce(&Match) {
        if let Ret::Pass(_, v) = &self { f(v); }
        self
    }

    #[inline(always)]
    pub fn inspect_err<F>(self, f: F) -> Self where F: FnOnce(&Cx::Info<'a, Unit>) {
        match &self {
            Ret::Miss(info) | Ret::Fail(info) => f(info),
            _ => {}
        }
        self
    }

    #[inline(always)]
    pub fn inspect_miss<F>(self, f: F) -> Self where F: FnOnce(&Cx::Info<'a, Unit>) {
        if let Ret::Miss(info) = &self { f(info); }
        self
    }

    #[inline(always)]
    pub fn inspect_fail<F>(self, f: F) -> Self where F: FnOnce(&Cx::Info<'a, Unit>) {
        if let Ret::Fail(info) = &self { f(info); }
        self
    }
}

impl<'a, Unit, Match, Cx: Ctx> SubRet<'a, Unit, Match, Cx> {
    pub const fn is_pass(&self) -> bool { matches!(self, Self::Pass(..)) }
    pub const fn is_miss(&self) -> bool { matches!(self, Self::Miss(..)) }

    #[track_caller]
    pub fn unwrap(self) -> (Cursor<'a, Unit>, Match) where Cx::Info<'a, Unit>: Display {
        match self {
            Self::Pass(cursor, value) => (cursor, value),
            Self::Miss(info) => panic!("called `SubRet::unwrap` on a `Miss`:\n\n{info}"),
        }
    }

    #[track_caller]
    pub fn expect(self, msg: &str) -> (Cursor<'a, Unit>, Match) where Cx::Info<'a, Unit>: Display {
        match self {
            Self::Pass(cursor, value) => (cursor, value),
            Self::Miss(info) => panic!("{msg}\n\n{info}"),
        }
    }

    pub fn ok(self) -> Option<Match> {
        match self { SubRet::Pass(_, value) => Some(value), _ => None }
    }

    pub fn option(self) -> Option<(Cursor<'a, Unit>, Match)> {
        match self { SubRet::Pass(cursor, value) => Some((cursor, value)), _ => None }
    }

    pub fn result(self) -> Result<(Cursor<'a, Unit>, Match), Cx::Info<'a, Unit>> {
        match self {
            SubRet::Pass(cursor, value) => Ok((cursor, value)),
            SubRet::Miss(info) => Err(info),
        }
    }

    pub fn unwrap_err(self) -> Cx::Info<'a, Unit> where Match: Debug, Cx::Info<'a, Unit>: Debug {
        match self {
            Self::Pass(..) => panic!("unwrap_err on a Pass: {:#?}", self),
            Self::Miss(info) => info,
        }
    }

    pub fn unwrap_miss(self) -> Cx::Info<'a, Unit> where Match: Debug, Cx::Info<'a, Unit>: Debug {
        match self {
            Self::Miss(info) => info,
            _ => panic!("unwrap_miss on a {:#?}", self),
        }
    }

    pub fn expect_err(self, msg: &str) -> Cx::Info<'a, Unit> where Match: Debug, Cx::Info<'a, Unit>: Debug {
        match self {
            Self::Pass(..) => panic!("{}: {:#?}", msg, self),
            Self::Miss(info) => info,
        }
    }

    pub fn expect_miss(self, msg: &str) -> Cx::Info<'a, Unit> where Match: Debug, Cx::Info<'a, Unit>: Debug {
        match self {
            Self::Miss(info) => info,
            _ => panic!("{}: {:#?}", msg, self),
        }
    }

    #[inline(always)]
    pub fn map<F, N>(self, f: F) -> SubRet<'a, Unit, N, Cx> where F: FnOnce(Match) -> N {
        match self {
            Self::Pass(c, v) => SubRet::Pass(c, f(v)),
            Self::Miss(info) => SubRet::Miss(info),
        }
    }

    #[inline(always)]
    pub fn map_err<F>(self, f: F) -> Self where F: FnOnce(Cx::Info<'a, Unit>) -> Cx::Info<'a, Unit> {
        match self {
            Self::Miss(info) => Self::Miss(f(info)),
            other => other,
        }
    }

    #[inline(always)]
    pub fn inspect<F>(self, f: F) -> Self where F: FnOnce(&Match) {
        if let Self::Pass(_, v) = &self { f(v); }
        self
    }

    #[inline(always)]
    pub fn inspect_err<F>(self, f: F) -> Self where F: FnOnce(&Cx::Info<'a, Unit>) {
        if let Self::Miss(info) = &self { f(info); }
        self
    }
}

