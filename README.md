Memoirs
=======

Memoization Functions for Rust

```rust
#![feature(phase)]
#![feature(unboxed_closures)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

#[phase(plugin)]
extern crate memoirs_mac;
extern crate memoirs;

#[phase(plugin)]
extern crate lazy_static;

static mut evaluations : int = 0;

//not thread safe
memoize!(
    fn double(i : int) -> int { 
        unsafe { evaluations += 1 };
        2 * i
    }
)

//protected by std::mutex::Mutex
memoize_sync!(
    fn triple(i : int) -> int {
        unsafe { evaluations += 1 };
        3 * i
    }
)

#[test]
fn basic_test() {    
    assert!(2 == double(1));
    assert!(4 == double(2));
    assert!(6 == double(3));

    assert!(unsafe { evaluations } == 3);

    assert!(2 == double(1));
    assert!(4 == double(2));
    assert!(6 == double(3));

    assert!(unsafe { evaluations } == 3);

    assert!(8 == double(4));
    assert!(8 == double(4));

    assert!(unsafe { evaluations } == 4);
}
```
