use clap::Parser;
use std::collections::HashMap;
use std::collections::HashSet;
use std::env;
use std::io;

mod goalwords;
mod morewords;

type WordSet = Vec<&'static str>;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// offset to count
    #[clap(short, long, default_value_t = 0)]
    count: usize,
}

#[allow(dead_code)]
fn print_type_of<T>(_: &T) {
    println!("{}", std::any::type_name::<T>())
}

#[allow(dead_code)]
fn count_chars(_column: usize) {
    let counted = goalwords::GOALWORDS
        .iter()
        //.skip(column)
        //.step_by(5)
        .flat_map(|w| w.chars())
        .filter(|c| c.is_ascii_lowercase())
        .fold(HashMap::with_capacity(26), |mut acc, c| {
            *acc.entry(c).or_insert(0) += 1;
            acc
        });

    let mut count_vec = counted.iter().collect::<Vec<(&char, &i32)>>();
    count_vec.sort_by(|a, b| b.1.cmp(a.1));
    count_vec.iter().for_each(|(c, x)| println!("{}:{}", c, x));
}

// List goal words
//fn list_goal_words() ->  Iter<'static, &'static str> {
//    goalwords::GOALWORDS.iter()
//}

//fn list_all_words() -> Iter<'static, &'static str>{
//    list_goal_words().chain(morewords::MOREWORDS.iter()).into_iter()
//}

// Create a list of all wordlewords that may be the goal words

struct WordleGame {
    goal_words: WordSet,
    word_set: HashSet<&'static str>,
}

impl WordleGame {
    pub fn new() -> Self {
        let mut x = goalwords::GOALWORDS.to_vec();
        let mut y = morewords::MOREWORDS.to_vec();
        x.append(&mut y);
        let mut all: HashSet<&'static str> = HashSet::new();

        for w in x {
            all.insert(w);
        }

        Self {
            goal_words: (goalwords::GOALWORDS.to_vec()),
            word_set: all,
        }
    }

    // Create a list of all wordlewords that may be the goal words
    fn get_goal_words(&self) -> &WordSet {
        &self.goal_words
    }

    fn get_relevant_words(guess_set: &WordSet, goal_set: &WordSet) -> WordSet {
        let mut bloom: Vec<bool> = vec![false; 256];

        for word in goal_set {
            for c in (*word).chars() {
                bloom[c as usize] = true;
            }
        }

        let mut relevant: WordSet = WordSet::new();

        for word in guess_set {
            for c in (*word).chars() {
                if bloom[c as usize] {
                    relevant.push(word);
                    break;
                }
            }
        }
        relevant
    }

    // Compare two words, and return how good the guess is relative to the goal.
    // Output is a string of five letters
    // ' ' means the guess is not in the goal word
    // 'Y' means the guess letter is in the goal word, but not in the right location.
    // 'G' means the guess letter is in the right location in the goal word.
    fn compare_words(goal: &str, guess: &str) -> String {
        let mut s: [char; 5] = [' ', ' ', ' ', ' ', ' '];

        let mut goal_chars: Vec<char> = goal.chars().collect();
        let guess_chars: Vec<char> = guess.chars().collect();

        // First pass. Mark the correct letters with a 'G'
        for i in 0..5 {
            if goal_chars[i] == guess_chars[i] {
                s[i] = 'G';
                goal_chars[i] = ' ' // Clear out this character so we don't match it again.
            }
        }

        // Second pass... Mark the guess letters that exist in the goal word but not in the right spot
        // as 'Y'
        for i in 0..5 {
            if s[i] == ' ' {
                let found = goal_chars.iter().enumerate().find_map(|(j, c)| {
                    if *c == guess_chars[i] {
                        Some(j)
                    } else {
                        None
                    }
                });
                if let Some(j) = found {
                    s[i] = 'Y';
                    goal_chars[j] = ' ';
                }
            }
        }

        s.iter().collect()
    }

    fn score(&self) -> (f64, String) {
        let word_set_count: f64 = self.word_set.len() as f64;
        println!("Word set count: {}", word_set_count);

        let mut max_score: f64 = 0.0;
        let mut max_fscore: f64 = 0.0;
        let mut max_word = String::from("");
        let min_of_max: usize = 10000;

        let all_words = goalwords::GOALWORDS
            .iter()
            .chain(morewords::MOREWORDS.iter());

        for possible_guess in all_words {
            // Calculate the clue sets and their size.
            let counted = &self
                .word_set
                .iter()
                .fold(HashMap::new(), |mut acc, possible_goal| {
                    *acc.entry(WordleGame::compare_words(possible_goal, possible_guess))
                        .or_insert(0) += 1;
                    acc
                });

            // Given the clue set, Calculate the Shannon entropy.
            let mut     fscore: f64 = counted
                .iter()
                .map(|(_key, value)| {
                    let v_c: f64 = f64::from(*value);
                    let f = word_set_count / v_c;
                    v_c * f.ln()
                })
                .sum::<f64>()
                / word_set_count;

            // Given a clue set, calculate it's size
            let word_count: f64 = counted.len() as f64;

            if (fscore == max_fscore && word_count > max_score)
                || (fscore == max_fscore
                    && word_count == max_score
                    && self.word_set.contains(possible_guess))
                || (fscore > max_fscore)
            {
                max_word = possible_guess.to_string();
                max_score = word_count;
                max_fscore = fscore;
            }
        }

        println!(
            "Guess... {}, {} {} {}",
            max_word, max_score, max_fscore, min_of_max
        );

        if max_score == 1.0 { max_fscore = 10.0 }

        (max_fscore, max_word)
    }

    fn remove(&mut self, guess: &str, clue: &str) {
        let guess_chars: Vec<char> = guess.chars().collect();
        let clue_chars: Vec<char> = clue.chars().collect();
        let mut remove_set = HashSet::new();

        for word in self.word_set.iter() {
            let mut word_chars: Vec<char> = word.chars().collect();
            let mut remove = false;

            for i in 0..5 {
                if clue_chars[i] == 'G' {
                    if guess_chars[i] == word_chars[i] {
                        word_chars[i] = ' '; // Don't match this letter again
                    } else {
                        // Remove words where the clue is green, but the letters don't match
                        remove = true;
                    }
                }

                if remove {
                    break;
                }
            }

            if !remove {
                for i in 0..5 {
                    if clue_chars[i] == 'Y' {
                        if guess_chars[i] == word_chars[i] {
                            // This should have been a 'G'
                            remove = true;
                            break;
                        }

                        // If the clue is Y then search for that letter.
                        // For Y, valid matches only happen when the match is not in the same position.
                        let found = word_chars.iter().enumerate().find_map(|(j, c)| {
                            if *c == guess_chars[i] {
                                Some(j)
                            } else {
                                None
                            }
                        });

                        if let Some(j) = found {
                            if j != i {
                                word_chars[j] = ' '; // Don't match this letter again.
                            } else {
                                remove = true; // This clue should have been 'G'
                            }
                        } else {
                            remove = true; // Didn't find the matching letter.
                        }
                    }

                    if remove {
                        break;
                    }
                }
            }

            if !remove {
                for i in 0..5 {
                    if clue_chars[i] == ' ' {
                        // If the clue is ' ' then that guess letter must not exist in the target.
                        let found = word_chars.iter().enumerate().find_map(|(j, c)| {
                            if *c == guess_chars[i] {
                                Some(j)
                            } else {
                                None
                            }
                        });

                        if let Some(_j) = found {
                            remove = true;
                        }

                        if remove {
                            break;
                        }
                    }
                }
            }

            if remove {
                remove_set.insert(*word);
            }
        }

        for removeable in &remove_set {
            self.word_set.remove(removeable);
        }
    }

    fn play_quordle() {
        let mut quordle: Vec<WordleGame> = Vec::new();
        quordle.push(WordleGame::new());
        quordle.push(WordleGame::new());
        quordle.push(WordleGame::new());
        quordle.push(WordleGame::new());

        //
        let mut recommend = String::from("slate");
        println!("Guess... slate");

        loop {
            let mut clue = String::new();

            println!("Enter csv clues...");
            io::stdin()
                .read_line(&mut clue)
                .expect("Failed to read line");

            let clues: Vec<&str> = clue.split(',').collect();

            for i in 0..=3 {
                if clues[i].len() > 0 {
                    quordle[i].remove(&recommend, clues[i]);
                }
            }

            let mut max_score: f64 = 0.0;
            let mut best_word: String = String::new();
            let mut f_next_score: f64;
            let mut f_next_word: String = String::new();

            for i in 0..=3 {
                if clues[i]. len() > 0 {
                    (f_next_score, f_next_word) = quordle[i].score();

                    if f_next_score > max_score {
                        max_score = f_next_score;
                        best_word = f_next_word;
                    }                    
                }
            }

            println!("Recommended: {}", best_word);
            recommend = best_word;
        } //loop

        //
    } //fn play_quordle
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args[1] == *"map" {
        println!("Words, sets, complex, max");
        for guess in goalwords::GOALWORDS {
            let counted = goalwords::GOALWORDS
                .iter()
                .fold(HashMap::new(), |mut acc, goal| {
                    *acc.entry(WordleGame::compare_words(goal, guess))
                        .or_insert(0) += 1;
                    acc
                });

            let max = counted.values().max();
            let set_count = counted.iter().fold(
                0,
                |acc, (_key, value)| if *value > 1 { acc + 1 } else { acc },
            );

            println!(
                "{}, {}, {}, {}",
                guess,
                counted.len(),
                set_count,
                max.unwrap()
            );
        }
    }

    if args[1] == *"log" {
        println!("Words, score");
        let all_words = goalwords::GOALWORDS
            .iter()
            .chain(morewords::MOREWORDS.iter());
        let _all_words_count: f64 = all_words.count() as f64;

        let all_words = goalwords::GOALWORDS
            .iter()
            .chain(morewords::MOREWORDS.iter());

        let goal_words = HashSet::from(["crack", "crazy", "cramp"]);
        let goal_words_count: f64 = goal_words.len() as f64;
        for guess in all_words {
            let counted = goal_words.iter().fold(HashMap::new(), |mut acc, goal| {
                *acc.entry(WordleGame::compare_words(goal, guess))
                    .or_insert(0) += 1;
                acc
            });

            // Calculate the Shannon entropy of this guess word.
            let score: f64 = counted
                .values()
                .map(|value| {
                    let v_c: f64 = f64::from(*value);
                    let f = v_c / goal_words_count;
                    v_c * f.ln()
                })
                .sum::<f64>()
                / goal_words_count;

            println!("{}, {}", guess, -score);
        }
    }

    if args[1] == *"play" {
        WordleGame::play_quordle();
    }

    if args[1] == *"deep" {
        let mut all = HashMap::new();
        let all_words = goalwords::GOALWORDS
            .iter()
            .chain(morewords::MOREWORDS.iter());
        for guess1 in all_words {
            println!("{}", guess1);
            let all_words2 = goalwords::GOALWORDS
                .iter()
                .chain(morewords::MOREWORDS.iter());
            for guess2 in all_words2 {
                let mut counted = HashMap::new();
                for goal in goalwords::GOALWORDS {
                    let filter1 = WordleGame::compare_words(goal, guess1);
                    let filter2 = WordleGame::compare_words(goal, guess2);
                    let mut filter = String::new();
                    filter.push_str(&filter1);
                    filter.push_str(&filter2);
                    //println!("{}", filter);
                    *counted.entry(filter).or_insert(0) += 1;
                }

                let mut wordpair = String::new();
                wordpair.push_str(guess1);
                wordpair.push_str(guess2);
                if counted.len() > 1000 {
                    println!("{} {}", wordpair, counted.len());
                    all.insert(wordpair, counted.len());
                }
            }
        }

        let mut count_vec = all.iter().collect::<Vec<(&String, &usize)>>();
        count_vec.sort_by(|(_, i1), (_, i2)| i1.cmp(i2));
        count_vec.iter().for_each(|(s, i)| println!("{}:{}", s, i));
    }

    if args[1] == *"scan" {
        let first_word = String::from(&args[2]);

        let mut counted = HashMap::new();

        for goal in goalwords::GOALWORDS {
            let filter = WordleGame::compare_words(goal, &first_word);
            *counted.entry(filter).or_insert(0) += 1;
        }

        let mut count_vec = counted.iter().collect::<Vec<(&String, &usize)>>();
        count_vec.sort_by(|(_, i1), (_, i2)| i1.cmp(i2));
        count_vec.iter().for_each(|(s, i)| println!("{}:{}", s, i));
    }
}

#[test]
fn it_works() {
    let s = compare_words("stern", "sueat");
    assert_eq!(s, String::from("G G Y"));

    let s = compare_words("stern", "clamp");
    assert_eq!(s, String::from("     "));

    let s = compare_words("stern", "stern");
    assert_eq!(s, String::from("GGGGG"));

    let s = compare_words("stern", "clamp");
    assert_eq!(s, String::from("     "));

    let s = compare_words("abcde", "edcba");
    assert_eq!(s, String::from("YYGYY"));
}

#[test]
fn remote_test() {
    let mut a = HashSet::from(["abcde", "abcdf", "abcdg"]);

    for word in a.iter() {
        println!("in {}", word.clone());
    }

    remove(&mut a, "iiiig", "    G");

    for word in a.iter() {
        println!("after {}", word.clone());
    }

    assert_eq!(a.contains("abcdg"), true);
    assert_eq!(a.contains("abcdf"), false);
}

#[test]
fn y_test() {
    let mut a = HashSet::from(["abcde", "fbcdf", "abcdg"]);

    remove(&mut a, "iifii", "  Y  ");

    assert_eq!(a.contains("fbcdf"), true);
    assert_eq!(a.contains("abcdg"), false);
}

#[test]
fn gy_test() {
    let mut a = HashSet::from(["abcae", "fbcdf", "abadg", "aiiba"]);

    remove(&mut a, "aiiia", "G   Y");

    assert_eq!(a.contains("fbcdf"), false);
    assert_eq!(a.contains("abcae"), true);
    assert_eq!(a.contains("abadg"), true);
    assert_eq!(a.contains("aiiba"), false);
}

#[test]
fn ggg_test() {
    let mut a = HashSet::from(["crack", "cramp"]);

    remove(&mut a, "crack", "GGG  ");

    assert_eq!(a.contains("crack"), false);
    assert_eq!(a.contains("cramp"), true);
}

#[test]
fn soare_await_test() {
    let a = compare_words("await", "soare");
    assert_eq!(a.eq("  G  "), true);
}

#[test]
fn soare_await_test2() {
    let mut b = HashSet::<&str>::new();
    b.insert("await");
    b.insert("admit");
    b.insert("avail");

    remove(&mut b, "soare", "  Y  ");
    assert_eq!(b.contains("admit"), true);
    assert_eq!(b.contains("await"), false);
}

#[test]
fn big_soare_test() {
    let str_await = String::from("await");
    let str_admit = String::from("admit");

    let mut word_set = goalwords::GOALWORDS
        .iter()
        .skip_while(|x| !str_await.eq(*x) && !str_admit.eq(*x))
        .fold(HashSet::<&str>::new(), |mut acc, word| {
            acc.insert(word);
            acc
        });

    remove(&mut word_set, "soare", "  Y  ");

    assert_eq!(word_set.contains("admit"), true);
    assert_eq!(word_set.contains("await"), false);
}
