extern crate rand;
extern crate rustbox;
use std::path::Path;
use std::convert::AsRef;
use std::fs::File;
use std::io::{self,BufReader,BufRead,Error};
use std::process::exit;
use std::collections::HashSet;
use rand::{Rng, seq::SliceRandom};
use rustbox::Event::KeyEvent;
use rustbox::Key::{self, Char, Esc};
use rustbox::{RustBox,Color};

fn load_dict<P: AsRef<Path>>(dict_path: P) -> Result<Vec<String>,io::Error> {
    let base_path = Path::new("/usr/share/dict");
    let path = base_path.join(dict_path);
    let mut file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let lines = reader.lines();
    Ok(
        lines
        .flat_map(|x| x.into_iter())
        .filter(|s| s.starts_with(char::is_alphabetic))
        .filter(|s| s.chars().all(char::is_lowercase))
        //.filter(|s| s.starts_with(char::is_uppercase))
        //.filter(|s| !s.ends_with("em"))
        //.filter(|s| !s.ends_with("es"))
        //.filter(|s| !s.ends_with("ds"))
        .collect()
    )
}

struct Letter {
    ch: char,
    revealed: bool
}

impl Letter {
    fn new(ch: char) -> Letter {
        Letter {
            ch: ch,
            revealed: false
        }
    }
    fn is_revealed(&self) -> bool { self.revealed }
    fn reveal(&mut self) { self.revealed = true }
    fn reveal_if_is(&mut self, guess: char) {
        if self.peek() == guess {
            self.reveal();
        }
    }
    fn peek(&self) -> char { self.ch }
    fn get(&self) -> Option<char> {
        if self.is_revealed() {
            Some(self.peek())
        }
        else { None }
    }
}

struct Game {
    word: Vec<Letter>,
    guesses_left: u32,
    guesses_made: HashSet<char>
}

impl Game {
    fn new(word: &str) -> Game {
        Game {
            word: word.chars().map(Letter::new).collect(),
            guesses_left: 8,
            guesses_made: HashSet::new()
        }
    }

    fn is_lost(&self) -> bool {self.guesses_left <= 0}
    fn is_won(&self) -> bool {self.word.iter().all(Letter::is_revealed)}
    fn is_running(&self) -> bool {!self.is_lost() && !self.is_won()}
    fn get_word(&self) -> Vec<Option<char>> {
        self.word.iter().map(Letter::get).collect()
    }
    fn peek_word(&self) -> String {
        self.word.iter().map(Letter::peek).collect()
    }
    fn word_contains(&self, ch: char) -> bool {
        self.word.iter().any(|letter| letter.peek() == ch)
    }
    fn guess(&mut self, guess: char) -> bool {
        if !self.is_running() || self.guesses_made.contains(&guess) {
            return false
        };
        let success = self.word_contains(guess);
        if success {
            for letter in &mut self.word {
                letter.reveal_if_is(guess)
            }
        } else {
            self.guesses_left -= 1;
        }
        self.guesses_made.insert(guess);
        success
    }
    fn guesses(&self) -> &HashSet<char> {
        &self.guesses_made
    }
    fn guesses_left(&self) -> u32 { self.guesses_left }
}

fn print_word(game: &Game, rustbox: &RustBox) {
    let word: String = game
        .get_word()
        .iter()
        .map(|letter| match letter {
            &None => format!("_ "),
            &Some(ch) => format!("{} ", ch)
        })
        .fold(String::new(), |mut buf, s| {
            buf.push_str(&s);
            buf
        });

    rustbox.print(
        1,
        1,
        rustbox::RB_BOLD,
        Color::White,
        Color::Black,
        &word
    );
}

fn print_guesses_made(game: &Game, rustbox: &RustBox) {
    let guesses: String = game.guesses().iter().cloned().collect();
    let msg = format!("Guesses: {}", &guesses);
    rustbox.print(
        1,
        3,
        rustbox::RB_BOLD,
        Color::White,
        Color::Black,
        &msg
    );
}

fn print_guesses_left(game: &Game, rustbox: &RustBox) {
    let msg = format!("Guesses left: {}", game.guesses_left());
    rustbox.print(
        1,
        5,
        rustbox::RB_BOLD,
        Color::White,
        Color::Black,
        &msg
    );
}

fn process_guess_input(game: &mut Game, guess: char, rustbox: &RustBox) {
    let is_valid = guess.is_alphabetic();
    let already_guessed = game.guesses().contains(&guess);
    let success = is_valid && game.guess(guess);
    let msg = match (is_valid, already_guessed, success) {
        (false, _, _) => format!("Invalid guess '{}'", guess),
        (_, true, _) => format!("Already guessed that!"),
        (_, _, true) => format!("Success!"),
        (_, _, false) =>  format!("There's no '{}' :(", guess)
    };
    rustbox.print(1, 7, rustbox::RB_BOLD, Color::White, Color::Black, &msg);
}

fn print_main(game: &Game, rustbox: &RustBox) {
    print_word(game, &rustbox);
    print_guesses_made(game, &rustbox);
    print_guesses_left(game, &rustbox);
}

fn main() {
    let dict = load_dict("american-english").ok().expect("can't open dict");
    let word = dict.choose(&mut rand::thread_rng()).expect("no words found");

    let ref mut game = Game::new(word);
    let rustbox = RustBox::init(Default::default()).ok().expect("rustbox initialization");

    print_main(game, &rustbox);
    rustbox.present();
    while game.is_running() {
        rustbox.clear();
        match rustbox.poll_event(false) {
            Ok(KeyEvent(Key::Char(guess))) =>
                process_guess_input(game, guess, &rustbox),
            Ok(KeyEvent(Key::Esc)) => {drop(rustbox); exit(0)}
            _ => rustbox.print(1, 7, rustbox::RB_BOLD, Color::White, Color::Black, "Eh.")
        }
        print_main(game, &rustbox);
        rustbox.present();
    }

    drop(rustbox);

    if game.is_won() {
        println!("Yay, you WIN!");
        println!("(The word was: '{}')", game.peek_word());
    }
    else {
        println!("He's dead, Jim! (You LOOSE!)");
        println!("(Btw, the word was: '{}')", game.peek_word());
    }

}
