Memoirs
=======

Memoization Functions for Rust

```rust
mem!(
  fn double(num : int) -> int { 
    println!("Evaluating...");
    num * 2 
  }
);

fn main() {
  println!("{}", double!(2)); //Evaluating... 4
  println!("{}", double!(2)); //4
  println!("{}", double!(2)); //4
  println!("{}", double!(2)); //4
  println!("{}", double!(10));//Evaluating... 20
}
```
