extern crate img_profile;

use img_profile::generate;

fn main() {
    generate(1000, 1000, "image.png".to_string()).unwrap();
}
