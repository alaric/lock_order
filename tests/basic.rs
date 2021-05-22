use lock_order::lock;
use std::sync::Mutex;

#[test]
fn simple_usage() {
    let lock1 = Mutex::new(1);
    let lock2 = Mutex::new(2);
    {
        lock!(mut lock2, mut lock1);
        *lock1 = 3;
        *lock2 = 4;
    }
    {
        lock!(mut lock2);
    }
}
