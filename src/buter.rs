use crate::SyncUnsafeCell;
use crossbeam_queue::ArrayQueue;
use parking_lot::lock_api::{Mutex, MutexGuard, RawMutex};
use std::ptr;

type Buf<T> = Vec<T>;

struct Buffers<T>(Box<[SyncUnsafeCell<Buf<T>>]>);

impl<T> Buffers<T> {
    fn new(size: usize) -> Self {
        Self((0..size).map(|_| SyncUnsafeCell::new(Buf::new())).collect())
    }

    unsafe fn get_unchecked(&self, index: usize) -> *mut Buf<T> {
        self.0.get_unchecked(index).get()
    }
}

pub struct Buter<T, R: RawMutex = parking_lot::RawMutex> {
    bufs: Buffers<T>,
    over: Mutex<R, Buf<T>>,
    queue: ArrayQueue<usize>,
}

impl<T: Unpin> Buter<T> {
    pub fn new() -> Self {
        Self::with_capacity(Self::DEFAULT_SIZE)
    }

    pub fn with_capacity(cap: usize) -> Self {
        let bufs = Buffers::new(cap);
        let queue = ArrayQueue::new(cap);
        for i in 0..cap {
            queue.push(i).unwrap();
        }

        Self {
            bufs,
            over: Mutex::new(Buf::new()),
            queue,
        }
    }
}

impl<T: Unpin, R: RawMutex> Buter<T, R> {
    const DEFAULT_SIZE: usize = 16;

    unsafe fn leak_buf(&self, place: usize) -> &mut Buf<T> {
        &mut *self.bufs.get_unchecked(place)
    }

    fn safe_leak_buf(&self) -> MutexGuard<'_, R, Buf<T>> {
        self.over.lock()
    }

    pub fn writer(&self) -> ButerWriter<'_, T, R> {
        if let Some(place) = self.queue.pop() {
            // SAFETY:
            unsafe {
                ButerWriter {
                    buf: BufRef::Free(self.leak_buf(place)),
                    que: Some(QueRef {
                        query: &self.queue,
                        place,
                    }),
                }
            }
        } else {
            ButerWriter {
                buf: BufRef::Lock(self.safe_leak_buf()),
                que: None,
            }
        }
    }
}

pub struct QueRef<'a> {
    pub(crate) query: &'a ArrayQueue<usize>,
    pub(crate) place: usize,
}

pub enum BufRef<'a, T, R: RawMutex> {
    Free(&'a mut Buf<T>),
    Lock(MutexGuard<'a, R, Buf<T>>),
}

impl<'a, T, R: RawMutex> BufRef<'a, T, R> {
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        match self {
            BufRef::Free(buf) => buf.get_mut(index),
            BufRef::Lock(buf) => buf.get_mut(index),
        }
    }

    pub unsafe fn set_len(&mut self, len: usize) {
        match self {
            BufRef::Free(buf) => buf.set_len(len),
            BufRef::Lock(buf) => buf.set_len(len),
        }
    }
}

pub struct ButerWriter<'a, T, R: RawMutex> {
    pub(crate) buf: BufRef<'a, T, R>,
    pub(crate) que: Option<QueRef<'a>>,
}

impl<'a, T: Unpin, R: RawMutex> IntoIterator for ButerWriter<'a, T, R> {
    type Item = T;
    type IntoIter = ButerIter<'a, T, R>;

    fn into_iter(self) -> Self::IntoIter {
        ButerIter::new(self)
    }
}

impl<'a, T, R: RawMutex> Extend<T> for ButerWriter<'a, T, R> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        match self.buf {
            BufRef::Free(ref mut buf) => {
                buf.extend(iter);
            }
            BufRef::Lock(ref mut buf) => {
                buf.extend(iter);
            }
        }
    }
}

impl<'a, T, R: RawMutex> Drop for ButerWriter<'a, T, R> {
    fn drop(&mut self) {
        if let Some(QueRef { place, query }) = self.que {
            let _ = query.push(place);
        }
    }
}

pub struct ButerIter<'a, T, R: RawMutex> {
    writer: ButerWriter<'a, T, R>,
    last: usize,
}

impl<'a, T, R: RawMutex> ButerIter<'a, T, R> {
    pub(crate) fn new(writer: ButerWriter<'a, T, R>) -> Self {
        Self { writer, last: 0 }
    }
}

impl<'a, T: Unpin, R: RawMutex> Iterator for ButerIter<'a, T, R> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let cur = self.writer.buf.get_mut(self.last);

        self.last += 1;
        cur.map(|t| unsafe {
            let new = ptr::read(t);
            ptr::write_bytes(t, 0, 1);
            new
        })
    }
}

impl<'a, T, R: RawMutex> Drop for ButerIter<'a, T, R> {
    fn drop(&mut self) {
        unsafe { self.writer.buf.set_len(0) }
    }
}
