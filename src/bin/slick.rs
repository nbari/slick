extern crate slick;

use slick::prompt;
use slick::precmd;

fn main() {
    let p = prompt::display();
    let c = precmd::display();
    print!("{}\n{}", c, p);
}
