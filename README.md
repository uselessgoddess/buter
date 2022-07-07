Do you have callbacks (and other difficult for iterators) and are you tired of `.collect()` them into `Vecs`? There is a
solution.

`Buter` - buffered iterator with fluid buffer, which is used as a place for all iterations

### Usage

```rust
fn callback_work<F>(mut f: F)
    where
        F: FnMut(usize),
{
    for i in 0..10_000 {
        f(i);
    }
}

use buter::Buter;

struct Worker {
    buf: Buter<usize>,
    // ...
}

impl Worker {
    fn work_impl(&self) -> impl Iterator<Item=usize> + '_ {
        let mut writer = self.buf.writer();
        callback_work(|i| writer.extend(Some(i)));
        writer.into_iter()
    }
}
```

### Benches
This is appropriate both with large results and with small ones
```rust
// test buter                  ... bench:          14 ns/iter (+/- 5)
// test vec_push               ... bench:         212 ns/iter (+/- 130)
// test vec_push_with_capacity ... bench:          54 ns/iter (+/- 32)
```
```rust
// buter                   time:   [2.7939 ms 2.8248 ms 2.8551 ms]
// vec_push                time:   [7.2415 ms 7.3449 ms 7.4483 ms]
// vec_push_with_capacity  time:   [4.3433 ms 4.3965 ms 4.4500 ms]
```
