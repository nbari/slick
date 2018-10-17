extern crate slick;

use slick::ctrl;

fn main() {
    let msg = ctrl::hello();
    println!("{}", msg);
}
