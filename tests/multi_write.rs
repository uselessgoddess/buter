use buter::Buter;
use crossbeam::thread;

#[test]
fn multi_writers() {
    let buf = Buter::with_capacity(1);

    let mut free_writer = buf.writer();
    free_writer.extend(0..10);

    // this write lock the thread because there are no places left for buffers
    let mut lock_writer = buf.writer();
    lock_writer.extend(20..30);

    // drop lock write will free the thread
    drop(lock_writer);

    let mut new_lock_writer = buf.writer();
    new_lock_writer.extend(30..40);

    // drop free lock free place for buffer
    drop(free_writer);

    let mut new_free_writer = buf.writer();
    new_free_writer.extend(10..20);

    assert!(new_free_writer.into_iter().eq(0..20));
    assert!(new_lock_writer.into_iter().eq(20..40));
}

#[test]
#[cfg(not(miri))]
fn multi_thread_writers() {
    let buf = Buter::with_capacity(1);

    let mut free_writer = buf.writer();
    free_writer.extend(0..10);
    assert!(free_writer.into_iter().eq(0..10));

    thread::scope(|s| {
        s.spawn(|_| {
            let mut lock_writer = buf.writer();
            lock_writer.extend(0..10);
            assert!(lock_writer.into_iter().eq(0..10));
        });

        s.spawn(|_| {
            let mut lock_writer = buf.writer();
            lock_writer.extend(0..10);
            assert!(lock_writer.into_iter().eq(0..10));
        });
    })
    .unwrap();
}
