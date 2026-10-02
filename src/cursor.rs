// TODO: this file hasn't change since like... 8 months ago, 90% of the methods aint used

impl<'a> Cursor<'a, u8> {
    pub fn skip_ws(mut self) -> Self {
        while let Some((new_cursor, &c)) = self.take_one() {
            if matches!(c, b' ' | b'\n' | b'\r' | b'\t') { self = new_cursor; }
            else { break; }
        }
        self
    }
}

impl<'a, T> std::fmt::Debug for Cursor<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Cursor[{}/{}]", self.index, self.source.len())
    }
}

impl<'a, T> Clone for Cursor<'a, T> {
    #[inline(always)]
    fn clone(&self) -> Self { *self }
}

impl<'a, T> Copy for Cursor<'a, T> {}


#[derive(PartialEq, Eq)]
pub struct Cursor<'a, T> {
    pub index: usize,
    pub source: &'a [T],
}

impl<'a, T> Cursor<'a, T> {
    // TODO: *should* this implement the trait?
    #[allow(clippy::should_implement_trait)]
    #[inline(always)]
    pub fn next(&mut self) -> Option<&'a T> {
        let item = self.source.get(self.index)?;
        self.index += 1;
        Some(item)
    }

    pub fn is_empty(&self) -> bool { self.index == self.source.len() }

    #[inline(always)]
    pub fn rest(&self) -> &'a [T] {
        debug_assert!(self.in_range(0), "cursor is already consumed");
        &self.source[self.index..]
    }

    #[inline(always)]
    pub fn may_advance(&self, amount: usize) -> Option<Self> {
        if self.in_range(amount) { Some(self.advance(amount)) } else { None }
    }

    #[inline(always)]
    pub fn advance(&self, amount: usize) -> Self {
        debug_assert!(self.in_range(amount), "can not advance beyond range of cursor");
        Self { index: self.index + amount, ..*self }
    }

    #[inline(always)]
    pub fn advance_upto(&self, amount: usize) -> Self {
        if self.in_range(amount) { return self.advance(amount) }
        Self { index: self.source.len(), ..*self }
    }

    #[inline(always)]
    pub fn may_slice(&self, amount: usize) -> Option<(Self, &'a [T])> {
        self.source.get(self.index..self.index+amount).map(|v| (self.advance(amount), v))
    }

    #[inline(always)]
    pub fn slice_between(&self, other: &Self) -> &'a [T] {
        &self.source[self.index..other.index]
    }

    #[inline(always)]
    pub fn window(&self, until: usize) -> &'a [T] {
        &self.source[self.index..until]
    }

    #[inline(always)]
    pub fn may_window(&self, until: usize) -> Option<&'a [T]> {
        self.source.get(self.index..self.index+until)
    }

    #[inline(always)]
    #[track_caller]
    pub fn peek(&self, amount: usize) -> Option<&'a [T]> {
        if !self.in_range(amount) { return None }
        Some(&self.source[self.index..self.index + amount])
    }

    #[inline(always)]
    pub fn take(&self, amount: usize) -> Option<(Self, &'a [T])> {
        if !self.in_range(amount) { return None }
        Some((self.advance(amount), &self.source[self.index..self.index + amount]))
    }

    #[inline(always)]
    pub fn take_one(self) -> Option<(Self, &'a T)> {
        self.source.get(self.index..self.index+1).map(|v| (self.advance(1), &v[0]))
        // if !self.in_range(1) { return None }
        // Some((self.advance(1), &self.source[self.index]))
    }

    #[inline(always)]
    pub fn in_range(&self, amount: usize) -> bool {
        self.index.saturating_add(amount) <= self.source.len()
    }
}

impl <'a,T>From<(&'a[T])>for(Cursor<'a,(T)>){
    fn from(source: &'a[T]) -> Self {
        Self {
            index: 0,source
        }
    }
}

impl <'a>From<(&'a str)>for(Cursor<'a,(u8)>){
    fn from(source: &'a str) -> Self {
        Self {
            index: 0,source: source.as_bytes()
        }
    }
}

impl <'a>From<(&'a String)>for(Cursor<'a,(u8)>){
    fn from(source: &'a String) -> Self {
        Self {
            index: 0,source: source.as_str().as_bytes()
        }
    }
}
