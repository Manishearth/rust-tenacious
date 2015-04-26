## rust-tenacious


This plugin warns when types marked `#[no_move]` are being moved.

Note that `#[no_move]` is transitive, any struct or enum containing a `#[no_move]` type
must be annotated as well. Similarly, any type with `#[no_move]` substitutions in its type parameters
(E.g. `Vec<Foo>` where `Foo` is `no_move`) will be treated as immovable.

Example:


```rust
#![plugin(tenacious)]
#![feature(custom_attribute, plugin)]

#[no_move]
#[derive(Debug)]
struct Foo;

fn main() {
    let x = Foo;
    let y = x; // warning
    bar(Some(y)); // warning   
}

fn bar(t: Option<Foo>) {
    match t {
        Some(foo) => { // warning
            println!("{:?}", foo)
        },
        _ => ()
    }

}

struct MoreFoo {
    foos: Vec<Foo> // warning
}

#[no_move]
struct MoreFoo2 {
    foos: Vec<Foo> // no warning
}
```

