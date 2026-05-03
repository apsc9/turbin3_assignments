# Assignment 1 - Guessing Game

## Objective

Build the Guessing Game project from the [Rust Book](https://doc.rust-lang.org/book/ch02-00-a-guessing-game-tutorial.html).

## About

A CLI guessing game written in Rust. The program generates a random number between 1 and 1000, and the player tries to guess it. After each guess, the game provides hints — too high, too low, or correct.

## How to Run

```bash
cd guessing_game
cargo run
```

## How It Works

1. A random secret number is generated (1–1000)
2. Player enters a guess
3. Game responds with "Try Something big!", "Try some smaller number!", or "You've hit the bull's eye!"
4. Loop continues until the correct number is guessed

## Dependencies

- `rand` — random number generation
