#![feature(unboxed_closures)]
#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![feature(quote)]

extern crate collections;
extern crate lazy_static;

// pub use lazy_static::lazy_static;

use std::collections::hash_map::HashMap;
use collections::hash::Hash;
use std::mem;

pub trait MemoizeFn<Args, Result> : MemoizeFnBackend<Args, Result> where Args : Clone , Result : Clone {
    fn mem_call(&self, args: Args) -> Result {
        let mut _self : &mut Self = unsafe { mem::transmute(self) };
        
        match _self.get_evaluation(&args) {
            Some(result) => return result,
            None => {
                let result = _self.call_underlying_fn(args.clone());
                _self.store_evaluation(args, result.clone());
                result
            }
        }
    }

    fn call_underlying_fn(&self, args : Args) -> Result;
}

pub trait MemoizeFnBackend<Args, Result> {
    fn get_evaluation(&mut self, args : &Args) -> Option<Result>;
    fn store_evaluation(&mut self, args : Args, result : Result);
}

//Different Backends
//-----------------------------------------------------------------------------

pub struct HashMapMemoizer<Args, Result> where Args : Eq + Hash {
    evals : HashMap<Args, Result>
}

impl<Args, Result> HashMapMemoizer<Args, Result> where Args : Eq + Hash {
    pub fn new() -> HashMapMemoizer<Args, Result> { 
        HashMapMemoizer {
            evals : HashMap::new()
        }
    }
}

impl<Args, Result> MemoizeFnBackend<Args, Result> for HashMapMemoizer<Args, Result> where Args : Eq + Hash, Result : Clone {
    fn get_evaluation(&mut self, args : &Args) -> Option<Result>{
        match self.evals.get(args) {
            Some(result) => Some(result.clone()),
            None => None
        }
    }

    fn store_evaluation(&mut self, args : Args, result : Result) {
        self.evals.insert(args, result);
    }   
}

//test
//---------------------------------------------------------------------------
// fn _double_(i : int) -> int {
//     2 * i
// }

// type _double_mem_func = memoirs::HashMapMemoizer<(int,), int>;

// impl _double_mem_func {
//     fn new() -> _double_mem_func {
//         memoirs::HashMapMemoizer::new() 
//     }
// }

// impl MemoizeFn<(int,), int> for _double_mem_func {
//     fn call_underlying_fn(&self, args : (int,)) -> int {
//         _double_.call(args)
//     }
// }

// impl Fn<(int,), int> for _double_mem_func {
//     extern "rust-call" fn call(&self, args: (int,)) -> int {
//         self.mem_call(args)
//     }
// }

// fn main() {
//     let double = _double_mem_func::new();
//     println!("{}", double(5));
// }

// lazy_static!{ static ref double : _double_mem_func = _double_mem_func::new() }

// impl Fn<(int,), int> for double {
//     extern "rust-call" fn call(&self, args: (int,)) -> int {
//         (*self)(args)
//     }
// }