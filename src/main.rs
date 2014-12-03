#![feature(macro_rules)]
#![feature(unboxed_closures)]
#![feature(phase)]

#[phase(plugin)]
extern crate lazy_static;
extern crate collections;

use std::collections::hash_map::HashMap;
use collections::hash::Hash;
use std::mem;

struct MemFunc<Args, Result> 
    where Args : Clone + Eq + Hash, Result : Clone {
    evals : HashMap<Args, Result>
}

//this type could be mangled however
type DoubleMemFunc = MemFunc<(int,), int>;

impl DoubleMemFunc {
    fn new() -> DoubleMemFunc {
        DoubleMemFunc { evals : HashMap::new() }
    }

    fn call(&self, args : (int,)) -> int {
      let _self : &mut DoubleMemFunc = unsafe { mem::transmute(self) };
      _self.call_mut(args)
    }
}

impl FnMut<(int,), int> for DoubleMemFunc {
    extern "rust-call" fn call_mut(&mut self, args: (int,)) -> int {
        //see if you have gotten these args already
        match self.evals.find(&args) {
            Some(result) => return result.clone(),
            None => {}
        };

        //run the func for the first time and cache the results
        println!("evaluating one time");
        let result = _double_.call(args);
        self.evals.insert(args.clone(), result.clone());
        result
    }
}

fn _double_(num : int) -> int { num * 2 }

lazy_static! {
  static ref double : DoubleMemFunc = DoubleMemFunc::new();
}

macro_rules! double(
  ($arg:expr) => (
    double.call(($arg,))  
  );
  ($($arg:expr),*) => ( 
    double.call(($($arg),*))
  );
)

//Now we just need a macro to use
//mem!(fn double(num : int) -> int { num * 2 });


fn main() {
    println!("{}", double!(2));
    println!("{}", double!(2));
    println!("{}", double!(2));
    println!("{}", double!(2));
    println!("{}", double!(10));
}
