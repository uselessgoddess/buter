use crossbeam_queue::ArrayQueue;
use parking_lot::{Mutex, MutexGuard};
use std::ptr;

type Buf<T> = Vec<T>;

struct Buffers<T>(Box<[Buf<T>]>);

unsafe fn leak_ref<'a, T>(t: &T) -> &'a mut T {
    &mut *(t as *const T as *mut T)
}

impl<T> Buffers<T> {
    fn new(size: usize) -> Self {
        Self((0..size).map(|_| Buf::new()).collect())
    }

    unsafe fn get_unchecked(&self, index: usize) -> &mut Buf<T> {
        leak_ref(&self.0[index])
    }
}

pub struct Buter<T> {
    bufs: Buffers<T>,
    over: Mutex<Buf<T>>,
    queue: ArrayQueue<usize>,
}

impl<T: Unpin> Buter<T> {
    const DEFAULT_SIZE: usize = 16;

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

    unsafe fn leak_buf(&self, place: usize) -> &mut Buf<T> {
        self.bufs.get_unchecked(place)
    }

    fn safe_leak_buf(&self) -> MutexGuard<'_, Buf<T>> {
        self.over.lock()
    }

    pub fn writer(&self) -> ButterWriter<'_, T> {
        if let Some(place) = self.queue.pop() {
            // SAFETY:
            unsafe {
                ButterWriter {
                    buf: BufRef::Free(self.leak_buf(place)),
                    que: Some(QueRef {
                        query: &self.queue,
                        place,
                    }),
                }
            }
        } else {
            ButterWriter {
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

pub enum BufRef<'a, T> {
    Free(&'a mut Buf<T>),
    Lock(MutexGuard<'a, Buf<T>>),
}

impl<'a, T> BufRef<'a, T> {
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

pub struct ButterWriter<'a, T> {
    pub(crate) buf: BufRef<'a, T>,
    pub(crate) que: Option<QueRef<'a>>,
}

impl<'a, T: Unpin> IntoIterator for ButterWriter<'a, T> {
    type Item = T;
    type IntoIter = ButerIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        ButerIter::new(self)
    }
}

impl<'a, T> Extend<T> for ButterWriter<'a, T> {
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

impl<'a, T> Drop for ButterWriter<'a, T> {
    fn drop(&mut self) {
        if let Some(QueRef { place, query }) = self.que {
            let _ = query.push(place);
        }
    }
}

pub struct ButerIter<'a, T> {
    writer: ButterWriter<'a, T>,
    last: usize,
}

impl<'a, T> ButerIter<'a, T> {
    pub(crate) fn new(writer: ButterWriter<'a, T>) -> Self {
        Self { writer, last: 0 }
    }
}

impl<'a, T: Unpin> Iterator for ButerIter<'a, T> {
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

impl<'a, T> Drop for ButerIter<'a, T> {
    fn drop(&mut self) {
        unsafe { self.writer.buf.set_len(0) }
    }
}
