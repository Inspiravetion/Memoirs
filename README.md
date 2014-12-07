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

static mut count : int = 0;

memoize!(
    fn double(i : int) -> int { 
        unsafe { count += 1 };
        2 * i
    }
)

#[test]
fn basic_test() {    
    assert!(2 == double(1));
    assert!(4 == double(2));
    assert!(6 == double(3));

    assert!(unsafe { count } == 3);

    assert!(2 == double(1));
    assert!(4 == double(2));
    assert!(6 == double(3));

    assert!(unsafe { count } == 3);

    assert!(8 == double(4));
    assert!(8 == double(4));

    assert!(unsafe { count } == 4);
}
```
