#![feature(plugin, custom_attribute)]
#![plugin(tenacious)]

#![deny(moved_no_move)]
#![allow(dead_code, unused)]

use std::mem;

// Test that boxes are allowed to contain no_move types

fn main() {
    let (x_box, addr_box) = box_move();
    let (x_ref, addr_ref) = ref_move();
    let (x_ptr, addr_ptr) = ptr_move();
    assert_eq!(addr_box, &*x_box as *const Foo as usize);
    assert_eq!(addr_ref, x_ref as *const Foo as usize);
    assert_eq!(addr_ptr, x_ptr as usize);
    assert_eq!(x_box.v, 1);
    assert_eq!(x_ref.v, 2);
    assert_eq!(unsafe { (*x_ptr).v }, 3);
}

fn box_move() -> (Box<Foo>, usize) {
    let x = Box::new(Foo { v: 1 });
    let addrx = &*x as *const Foo as usize;
    let ret = x;
    (ret, addrx)
}

fn ref_move() -> (&'static Foo, usize) {
    let x = Box::new(Foo { v: 2 });
    let addrx = &*x as *const Foo as usize;
    let ret = unsafe { mem::transmute(&*x) };
    mem::forget(x);
    (ret, addrx)
}

fn ptr_move() -> (*const Foo, usize) {
    let x = Box::new(Foo { v: 3 });
    let addrx = &*x as *const Foo as usize;
    let ret = &*x as *const Foo;
    mem::forget(x);
    (ret, addrx)
}


#[no_move]
struct Foo {
    v: usize,
}
