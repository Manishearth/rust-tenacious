#![feature(plugin, custom_attribute)]
#![plugin(tenacious)]

#![deny(moved_no_move)]
#![allow(dead_code, unused)]
fn main() {
    let x = Foo;
    bar(FooBar::Foo(x)); //~ ERROR #[no_move] type `Foo` moved
}

fn bar(x: FooBar) {
   match x {
        FooBar::Foo(foo) => { //~ ERROR #[no_move] type `Foo` moved
            let y = foo; //~ ERROR #[no_move] type `Foo` moved
            println!("{:?}", y)
        },
        FooBar::Bar => ()
   }
}

fn baz() {
    let mut x = Vec::new();
    x.push(Foo);
    let y = x; //~ ERROR #[no_move] type `std::vec::Vec<Foo>` moved
}

fn quux() -> Foo {
    let x = Foo;
    x //~ ERROR
}

fn test() {
    let x = quux();
    let y = x; //~ ERROR
}
#[derive(Debug)]
#[no_move]
struct Foo;

enum FooBar {
    Foo(Foo), //~ ERROR Structs and enums containing
    Bar
}

struct FooStruct<'a> {
    foo: Foo, //~ ERROR Structs and enums containing
    bar: u8,
    foovec: Vec<Foo>, //~ ERROR Structs and enums containing
    fooarr: [Foo; 5], //~ ERROR Structs and enums containing
    fooptr: &'a Foo // this is okay
}

#[no_move]
struct FooStruct2 {
    foo: Foo,
    bar: u8,
    foovec: Vec<Foo>,
    fooarr: [Foo; 5]
}

