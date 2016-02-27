#![feature(plugin, custom_attribute)]
#![plugin(tenacious)]

#![deny(moved_no_move)]
#![allow(dead_code, unused)]

// Test for permitting usage of `#[no_move]` types.

fn main() {
    let x = MovableFoo { v: Box::new(Foo) };
    let y = Movable { v: Box::new(Foo) };
    let a = a(x, y);
    b(a);
}

fn a(x: MovableFoo, y: Movable<Foo>) -> CollectedMovable<Foo> {
    let a = 1;
    let ret = CollectedMovable {
        v1: Box::new(Foo),
        v2: Box::new(Foo),
    };
    let b = 2;
    ret
}

fn b(x: CollectedMovable<Foo>) {}

#[allow(moved_no_move)]
struct MovableFoo {
    v: Box<Foo>,
}

#[allow_movable_interior]
struct Movable<T> {
    v: Box<T>,
}

// CollectedMovable<Foo> is used so `#[allow(moved_no_move)]` isn't sufficient. 
#[allow_movable_interior]
struct CollectedMovable<T> {
    v1: Box<T>,
    v2: Box<Foo>,
}

#[no_move]
struct Foo;
