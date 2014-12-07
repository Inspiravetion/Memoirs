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

memoize_sync!(
    fn triple(i : int) -> int { 
        unsafe { count += 1 };
        3 * i
    }
)

#[test]
fn single_threaded_basic_test() {    
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

    unsafe { count = 0 };
}

#[test]
fn single_threaded_sync_test() {    
    assert!(3 == triple(1));
    assert!(6 == triple(2));
    assert!(9 == triple(3));

    assert!(unsafe { count } == 3);

    assert!(3 == triple(1));
    assert!(6 == triple(2));
    assert!(9 == triple(3));

    assert!(unsafe { count } == 3);

    assert!(12 == triple(4));
    assert!(12 == triple(4));

    assert!(unsafe { count } == 4);

    unsafe { count = 0 };
}