#![feature(test)]

extern crate rand;
extern crate test;

use rand::Rng;
use std::env;
use std::io;
use std::io::Read;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Instant;

mod crypt;

fn salt_replace(c: char) -> char {
    match c {
        '/' |
        '0' |
        '1' |
        '2' |
        '3' |
        '4' |
        '5' |
        '6' |
        '7' |
        '8' |
        '9' |
        'A' |
        'B' |
        'C' |
        'D' |
        'E' |
        'F' |
        'G' |
        'H' |
        'I' |
        'J' |
        'K' |
        'L' |
        'M' |
        'N' |
        'O' |
        'P' |
        'Q' |
        'R' |
        'S' |
        'T' |
        'U' |
        'V' |
        'W' |
        'X' |
        'Y' |
        'Z' |
        'a' |
        'b' |
        'c' |
        'd' |
        'e' |
        'f' |
        'g' |
        'h' |
        'i' |
        'j' |
        'k' |
        'l' |
        'm' |
        'n' |
        'o' |
        'p' |
        'q' |
        'r' |
        's' |
        't' |
        'u' |
        'v' |
        'w' |
        'x' |
        'y' |
        'z'  => c,
        ':'  => 'A',
        ';'  => 'B',
        '<'  => 'C',
        '='  => 'D',
        '>'  => 'E',
        '?'  => 'F',
        '@'  => 'G',
        '['  => 'a',
        '\\' => 'b',
        ']'  => 'c',
        '^'  => 'd',
        '_'  => 'e',
        '`'  => 'f',
        _    => '.',
    }
}

fn trip(pass: &str) -> Option<String> {
    let salt: String = pass.chars()
        .chain("H.".chars())
        .skip(1)
        .take(2)
        .map(salt_replace)
        .collect();

    crypt::crypt(pass, &salt)
}

struct Match {
    pass: String,
    trip: String,
}

fn try(pats: &[String]) -> Option<Match> {
    let pass: String = rand::thread_rng()
            .gen_ascii_chars()
            .take(9)
            .collect();

    trip(&pass).and_then(|t| {
        if pats.iter().any(|p| t.contains(p.as_str())) {
            Some(Match {
                pass: pass,
                trip: t,
            })
        } else {
            None
        }
    })
}

fn main() {
    let procs = if let Ok(v) = env::var("PROCS") {
        v.parse().unwrap()
    } else {
        1
    };

    let done = Arc::new(AtomicBool::new(false));
    let pats: Vec<String> = env::args().collect();
    let mut threads = Vec::with_capacity(8);
    let now = Instant::now();

    for _ in 0..procs {
        let done = done.clone();
        let pats = pats.clone();

        threads.push(
            thread::spawn(move || {
                let mut n: u64 = 0;

                while !done.load(Ordering::Relaxed) {
                    if let Some(m) = try(&pats) {
                        println!("#{} => {}", m.pass, m.trip);
                    }

                    n += 1;
                }

                n
            })
        );
    }

    io::stdin().bytes().next();
    done.store(true, Ordering::Relaxed);

    let n = threads.into_iter().fold(0, |acc, t| acc + t.join().unwrap());
    let tps = n / now.elapsed().as_secs();
    println!("{} tripcodes/second", tps);
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    #[bench]
    fn bench_trip(b: &mut Bencher) {
        b.iter(|| trip("foo"));
    }

    #[bench]
    fn bench_try(b: &mut Bencher) {
        let pats = vec![String::from("foo")];
        b.iter(|| try(&pats));
    }
}
