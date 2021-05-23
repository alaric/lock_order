## Lock ordering macro.

### Raison D'Être

This crate provides a simple lock ordering procmacro for ensuring a deterministic locking order,
which is useful as a pattern to prevent deadlocks between fine-grain mutex use.

It also serves to remove the `unwrap()` of panic-propagation between threads in the case of
poisoned locks. This is my favoured approach for handling an already panicking program, but
makes it difficult to find other non-valid usages of `unwrap()` in the code.

### Basic Usage

- The `mut` is optional based on if you want mutability, but must be prior to the identifier
- The identifier can be multiple field lookups, ie `self.locks.connections` and will result in a
bound variable `connections` as the last part of the full identifier.
- There can be one or more locks provided, separated by `,`, they will be ordered
lexicographially by the bound variable name.

Thus an example like this:
```rust
use lock_order::lock;
use std::sync::Mutex;

let lock1 = Mutex::new(1);
let lock2 = Mutex::new(2);
let lock3 = Mutex::new(3);
{
    lock!(mut lock2, lock3, mut lock1);
    *lock1 = 3 + *lock3;
    *lock2 = 4 + *lock3;
}
```

Would expand to:

```rust
use lock_order::lock;
use std::sync::Mutex;

let lock1 = Mutex::new(1);
let lock2 = Mutex::new(2);
let lock3 = Mutex::new(3);
{
    let (mut lock1, mut lock2, lock3) = (lock1.lock().unwrap(), lock2.lock().unwrap(),
    lock3.lock().unwrap());
    *lock1 = 3 + *lock3;
    *lock2 = 4 + *lock3;
}
```

### Future direction

- Support for RwLock
- Support for bare non-poisoning locks such as `parking_lot`, which don't require `unwrap()`.
