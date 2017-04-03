#![feature(test)]

extern crate test;
extern crate rand;

use rand::Rng;
use std::env;
use std::thread;

mod crypt;

fn salt_replace(c: char) -> char {
    match c {
        '/' | '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' | 'A' | 'B' | 'C' |
        'D' | 'E' | 'F' | 'G' | 'H' | 'I' | 'J' | 'K' | 'L' | 'M' | 'N' | 'O' | 'P' | 'Q' |
        'R' | 'S' | 'T' | 'U' | 'V' | 'W' | 'X' | 'Y' | 'Z' | 'a' | 'b' | 'c' | 'd' | 'e' |
        'f' | 'g' | 'h' | 'i' | 'j' | 'k' | 'l' | 'm' | 'n' | 'o' | 'p' | 'q' | 'r' | 's' |
        't' | 'u' | 'v' | 'w' | 'x' | 'y' | 'z' => c,
        ':' => 'A',
        ';' => 'B',
        '<' => 'C',
        '=' => 'D',
        '>' => 'E',
        '?' => 'F',
        '@' => 'G',
        '[' => 'a',
        '\\' => 'b',
        ']' => 'c',
        '^' => 'd',
        '_' => 'e',
        '`' => 'f',
        _ => '.',
    }
}

struct Match {
    pass: String,
    trip: String,
}

fn try<I: Iterator<Item = String>>(mut pats: I) -> Option<Match> {
    let pass: String = rand::thread_rng().gen_ascii_chars().take(9).collect();

    let salt: String = pass.chars()
        .chain("H.".chars())
        .skip(1)
        .take(2)
        .map(salt_replace)
        .collect();

    let trip = crypt::crypt(&pass, &salt);

    pats.find(|p| trip.contains(p.as_str()))
        .and_then(|_| {
                      Some(Match {
                               pass: pass,
                               trip: trip,
                           })
                  })
}

fn main() {
    let procs = env::var("PROCS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(1);

    for t in (0..procs).map(|_| {
                                thread::spawn(move || loop {
                                                  if let Some(m) = try(env::args()) {
                                                      println!("#{} => {}", m.pass, m.trip);
                                                  }
                                              })
                            }) {
        t.join().unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::iter;
    use test::Bencher;

    #[bench]
    fn bench_try(b: &mut Bencher) {
        b.iter(|| try(iter::once("foo".to_string())))
    }
}
