// Refactored Rust Code for Wordle Solver

use clap::Parser;
use std::collections::{HashMap, HashSet};
use std::env;
use std::io;
use std::iter::FromIterator;

mod goalwords;
mod morewords;

// Command-line arguments structure
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Offset to count
    #[clap(short, long, default_value_t = 0)]
    count: usize,
}

// Utility Functions
fn print_type_of<T>(_: &T) {
    println!("{}", std::any::type_name::<T>());
}

fn read_input(prompt: &str) -> String {
    println!("{}", prompt);
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

fn count_chars() {
    let counted = goalwords::GOALWORDS
        .iter()
        .flat_map(|w| w.chars())
        .filter(|c| c.is_ascii_lowercase())
        .fold(HashMap::with_capacity(26), |mut acc, c| {
            *acc.entry(c).or_insert(0) += 1;
            acc
        });

    let mut count_vec: Vec<(&char, &i32)> = counted.iter().collect();
    count_vec.sort_by(|a, b| b.1.cmp(a.1));
    count_vec.iter().for_each(|(c, x)| println!("{}: {}", c, x));
}

// WordleGame Struct and Implementation
struct WordleGame {
    word_set: HashSet<&'static str>,
}

impl WordleGame {
    pub fn new() -> Self {
        let all: HashSet<&'static str> = goalwords::GOALWORDS.iter().cloned().collect();
        Self { word_set: all }
    }

    fn compare_words(goal: &str, guess: &str) -> String {
        let mut feedback_result = [' '; 5];
        let mut goal_chars: Vec<char> = goal.chars().collect();
        let guess_chars: Vec<char> = guess.chars().collect();

        for i in 0..5 {
            if goal_chars[i] == guess_chars[i] {
                feedback_result[i] = 'G';
                goal_chars[i] = ' ';
            }
        }

        for i in 0..5 {
            if feedback_result[i] == ' ' {
                if let Some(pos) = goal_chars.iter().position(|&c| c == guess_chars[i]) {
                    feedback_result[i] = 'Y';
                    goal_chars[pos] = ' ';
                }
            }
        }

        feedback_result.iter().collect()
    }

    fn calculate_entropy(&self, guess: &str) -> (f64, usize) {
        let clue_counts = self.word_set.iter().fold(HashMap::new(), |mut acc, goal| {
            *acc.entry(Self::compare_words(goal, guess)).or_insert(0) += 1;
            acc
        });

        let word_set_count = self.word_set.len() as f64;
        let entropy: f64 = clue_counts
            .values()
            .map(|&count| {
                let probability = count as f64 / word_set_count;
                -probability * probability.ln()
            })
            .sum();

        (entropy, clue_counts.len())
    }

    fn find_best_guess(&self) -> String {
        let mut best_word = "";
        let mut highest_entropy = 0.0;

        for &guess in goalwords::GOALWORDS.iter().chain(morewords::MOREWORDS.iter()) {
            let (entropy, _) = self.calculate_entropy(guess);
            if entropy > highest_entropy {
                highest_entropy = entropy;
                best_word = guess;
            }
        }

        best_word.to_string()
    }

    fn remove_invalid_words(&mut self, guess: &str, clue: &str) {
        let guess_chars: Vec<char> = guess.chars().collect();
        let clue_chars: Vec<char> = clue.chars().collect();
        self.word_set.retain(|&word| {
            let mut word_chars: Vec<char> = word.chars().collect();

            for i in 0..5 {
                match clue_chars[i] {
                    'G' if guess_chars[i] != word_chars[i] => return false,
                    'Y' if guess_chars[i] == word_chars[i] => return false,
                    ' ' if word_chars.contains(&guess_chars[i]) => return false,
                    _ => (),
                }
            }

            true
        });
    }
}

// Main Program Flow
fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    let args: Vec<String> = env::args().collect();

    match args.get(1).map(String::as_str) {
        Some("play") => play_quordle(),
        _ => println!("Invalid command. Use 'play' for Quordle mode."),
    }
}

fn play_quordle() {
    let mut games: Vec<WordleGame> = vec![WordleGame::new(); 4];

    loop {
        let current_guess = read_input("Enter your current guess:");
        let feedback = read_input("Enter feedback for each game (comma-separated):");
        let clues: Vec<&str> = feedback.split(',').collect();

        for (i, game) in games.iter_mut().enumerate() {
            if let Some(&clue) = clues.get(i) {
                game.remove_invalid_words(&current_guess, clue);
            }
        }

        let recommended_guess = games
            .iter()
            .map(|game| game.find_best_guess())
            .max_by(|a, b| a.len().cmp(&b.len()))
            .unwrap_or_else(|| "error".to_string());

        println!("Recommended next guess: {}", recommended_guess);
    }
}
