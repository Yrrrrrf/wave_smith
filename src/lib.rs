pub fn some_fn() {println!("Some function");}

pub fn another_fn() {println!("Another function");}



pub mod morse;


#[cfg(test)]
mod tests {

    #[test]
    fn some_test() {assert_eq!((2_i32.pow(3))-4, 4);}

    #[test]
    fn some_test_2() {assert_eq!((2_i32.pow(3))-4, 4);}
}

