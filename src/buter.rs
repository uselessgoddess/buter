use crossbeam_queue::ArrayQueue;
use lock_api::{MappedRwLockReadGuard, MappedRwLockWriteGuard, RwLockReadGuard, RwLockWriteGuard};
use parking_lot::{RawRwLock, RwLock};
use std::ptr;

type Buf<T> = Vec<T>;
type MappedRwLockRead<'a, T> = MappedRwLockReadGuard<'a, RawRwLock, T>;
type MappedRwLockWrite<'a, T> = MappedRwLockWriteGuard<'a, RawRwLock, T>;

struct Buffers<T>(Box<[Buf<T>]>);

unsafe fn leak_ref<'a, T>(t: &T) -> &'a mut T {
    &mut *(t as *const T as *mut T)
}

impl<T> Buffers<T> {
    fn new(size: usize) -> Self {
        Self((0..size + 1).map(|_| Buf::new()).collect())
    }

    unsafe fn get_unchecked(&self, index: usize) -> &mut Buf<T> {
        leak_ref(&self.0[index])
    }

    fn size(&self) -> usize {
        self.0.len() - 1
    }

    fn guard_zone(&mut self) -> &mut Buf<T> {
        &mut self.0[self.size()]
    }
}

pub struct Buter<T> {
    bufs: RwLock<Buffers<T>>,
    queue: ArrayQueue<usize>,
}

impl<T: Unpin> Buter<T> {
    const SIZE: usize = 32;

    pub fn new() -> Self {
        let bufs = Buffers::new(Self::SIZE);
        let queue = ArrayQueue::new(Self::SIZE);
        for i in 0..Self::SIZE {
            queue.push(i).unwrap();
        }

        Self {
            bufs: RwLock::new(bufs),
            queue,
        }
    }

    unsafe fn leak_buf(&self, place: usize) -> MappedRwLockRead<'_, Buf<T>> {
        let bufs = self.bufs.read();
        RwLockReadGuard::map(bufs, |bufs| bufs.get_unchecked(place))
    }

    fn safe_leak_buf(&self) -> MappedRwLockWrite<'_, Buf<T>> {
        let bufs = self.bufs.write();
        RwLockWriteGuard::map(bufs, |bufs| bufs.guard_zone())
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
    Free(MappedRwLockRead<'a, Buf<T>>),
    Lock(MappedRwLockWrite<'a, Buf<T>>),
}

impl<'a, T> BufRef<'a, T> {
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        unsafe {
            match self {
                BufRef::Free(buf) => leak_ref(&**buf).get_mut(index),
                BufRef::Lock(buf) => buf.get_mut(index),
            }
        }
    }

    pub unsafe fn set_len(&mut self, len: usize) {
        match self {
            BufRef::Free(buf) => leak_ref(&**buf).set_len(len),
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
            BufRef::Free(ref mut buf) => unsafe {
                leak_ref(&**buf).extend(iter);
            },
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
