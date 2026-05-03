use std::io;
use rand::Rng;
use std::cmp::Ordering;

fn main() {
    println!("Welcome to the Guessing Game");

    let secret_number = rand::thread_rng().gen_range(1..=1000);

    loop {
        println!("Please enter your guess");

        let mut guess = String::new();

        io::stdin()
            .read_line(&mut guess)
            .expect("Unfortunately failed to read the line");

        let guess: u32 = match guess.trim().parse() {
            Ok(num) => num,
            Err(_) => continue,
        };

        println!("Your guess: {}", guess);

        match guess.cmp(&secret_number) {
            Ordering::Less => println!("Try Something big!"),
            Ordering::Greater => println!("Try some smaller number!"),
            Ordering::Equal => {
                println!("You've hit the bull's eye!");
                break;
            }
        }
    }
    
}
