#![feature(plugin, custom_attribute)]
#![plugin(tenacious)]

#![deny(moved_no_move)]
#![allow(dead_code, unused)]

// Test for permitting usage of `#[no_move]` types.

use std::sync::Arc;
use std::rc::Rc;

fn main() {
    let x = MovableFoo { v: Foo };
    let y = Movable { v: Foo };
    let a = a(x, y);
    b(a);
}

fn a(x: MovableFoo, y: Movable<Foo>) -> CollectedMovable<Foo> {
    let a = 1;
    let ret = CollectedMovable {
        v1: Foo,
        v2: Foo,
    };
    let b = 2;
    ret
}

fn b(x: CollectedMovable<Foo>) {}

#[allow(moved_no_move)]
struct MovableFoo {
    v: Foo,
}

#[allow_movable_interior]
struct Movable<T> {
    v: T,
}

// CollectedMovable<Foo> is used so `#[allow(moved_no_move)]` isn't sufficient. 
#[allow_movable_interior]
struct CollectedMovable<T> {
    v1: T,
    v2: Foo,
}

struct Indirection(Arc<Foo>, Rc<Foo>, Box<Foo>,);

fn indirect() -> Indirection {
    let a = 1;
    let ret = Indirection(Arc::new(Foo), Rc::new(Foo), Box::new(Foo));
    return ret;
}

#[no_move]
struct Foo;
