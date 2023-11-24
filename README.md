# MuTeX GROUP

[![Crates.io version](https://img.shields.io/crates/v/mtxgroup.svg)](https://crates.io/crates/mtxgroup)

## Introduction

**M**u**T**e**X** **GROUP** can be used to lock all mutexes at the same time.

## Example

```rust
static MUTEX_0: Mutex<usize> = Mutex::new(0);
static MUTEX_1: Mutex<usize> = Mutex::new(1);
static MUTEX_2: Mutex<usize> = Mutex::new(2);

let mutexes = vec![&MUTEX_0, &MUTEX_1, &MUTEX_2];
let mtxgroup = MutexGroup::new(mutexes.into_iter());
let guard = mtxgroup.lock(); // Locks all mutexes at the same time

for mutex in guard.iter() {
    // Do something with the mutexes
}

for mut mutex in guard.into_iter() { // `into_iter` consumes the guard
    // Do something with the mutexes
}
```

## Features

- Compliable with `no_std`
  - requires `spin` crate

## License

Licensed under [MIT license](./LICENSE)
