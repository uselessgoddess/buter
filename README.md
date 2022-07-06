Do you have callbacks (and other difficult for iterators) and are you tired of `.collect()` them into `Vecs`? There is a
solution.

`Buter` - buffered iterator with fluid buffer, which is used as a place for all iterations

### Usage

```rust
struct Worker {
    // ...
}

impl Worker {
    fn work_impl(&mut self) -> impl Iterator<Item=usize> + '_ {
        callback_work(|i| self.buf.extend_one(i));
        self.buf.iter()
    }
}

struct YourType {
    worker: Mutex<Worker>,
    // ...
}

impl YourType {
    pub fn work(&self) -> impl Iterator<Item=usize> + '_ {
        self.worker.lock().work_impl()
    }
}
```

### Benches
This is appropriate both with large results and with small ones
```
test buter                  ... bench:          14 ns/iter (+/- 5)
test vec_push               ... bench:         212 ns/iter (+/- 130)
test vec_push_with_capacity ... bench:          54 ns/iter (+/- 32)
```
```
buter                   time:   [4.7527 ms 4.8466 ms 4.9428 ms]
vec_push                time:   [9.6938 ms 9.8887 ms 10.086 ms]
vec_push_with_capacity  time:   [6.5350 ms 6.6588 ms 6.7789 ms]

```
