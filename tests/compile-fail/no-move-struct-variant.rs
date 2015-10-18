#![feature(plugin, custom_attribute)]
#![plugin(tenacious)]

#![deny(moved_no_move)]
#![allow(dead_code, unused)]

// Test to ensure that struct variants don't cause panics

fn main() {
}

enum FooBar {
    Foo {foo: Foo}, //~ ERROR Structs and enums containing
    Bar
}


#[no_move]
struct Foo;