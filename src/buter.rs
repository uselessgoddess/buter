use std::ptr;

pub struct Buter<T> {
    buf: Vec<T>,
}

impl<T: Unpin> Buter<T> {
    pub const fn new() -> Self {
        Self { buf: Vec::new() }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buf: Vec::with_capacity(capacity),
        }
    }

    pub fn iter(&mut self) -> impl Iterator<Item = T> + '_ {
        ButerIter::new(&mut self.buf)
    }
}

impl<T> Extend<T> for Buter<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        self.buf.extend(iter);
    }
}

pub struct ButerIter<'a, T> {
    buf: &'a mut Vec<T>,
    last: usize,
}

impl<'a, T> ButerIter<'a, T> {
    pub(crate) fn new(buf: &'a mut Vec<T>) -> Self {
        Self { buf, last: 0 }
    }
}

impl<'a, T: Unpin> Iterator for ButerIter<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let cur = self.buf.get_mut(self.last);
        self.last += 1;
        cur.map(|t| unsafe {
            let new = ptr::read(t);
            ptr::write_bytes(t, 0, 1);
            new
        })
    }
}

impl<'a, T> Drop for ButerIter<'a, T> {
    fn drop(&mut self) {
        unsafe { self.buf.set_len(0) }
    }
}
