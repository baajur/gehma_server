#[macro_use]
extern crate diesel;
#[macro_use]
extern crate serde_derive;

pub mod models;
pub mod utils;
pub mod errors;
pub mod schema;
pub mod lvl;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
