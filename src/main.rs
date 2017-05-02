#![feature(test)]

extern crate test;
extern crate rand;

use rand::Rng;
use std::env;
use std::thread;

mod trip;

struct Match {
    passwd: String,
    trip: String,
}

fn try<I: Iterator<Item = String>>(mut pats: I) -> Option<Match> {
    let passwd: String = rand::thread_rng().gen_ascii_chars().take(8).collect();
    let trip = trip::trip(&passwd);

    if trip.chars().next().unwrap() != '#' {
        pats.find(|p| trip.contains(p.as_str()))
            .and_then(
                |_| {
                    Some(
                        Match {
                            passwd: passwd,
                            trip: trip,
                        },
                    )
                },
            )
    } else {
        None
    }
}

fn main() {
    let procs = env::var("NUMBER_OF_PROCESSORS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(1);

    for t in (0..procs).map(
        |_| {
            thread::spawn(
                move || loop {
                    if let Some(m) = try(env::args()) {
                        println!("#{} => {}", m.passwd, m.trip);
                    }
                },
            )
        },
    ) {
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
