extern crate rand;

use rand::Rng;
use std::env;
use std::io::{self, Read};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Instant;

mod trip;

fn main() {
    let procs = env::var("NUMBER_OF_PROCESSORS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(1);

    let abort = Arc::new(AtomicBool::new(false));
    let now = Instant::now();

    let threads: Vec<_> = (0..procs)
        .map(|_| abort.clone())
        .map(|abort| {
            thread::spawn(move || {
                let mut count = 0;

                while !abort.load(Ordering::Relaxed) {
                    let passwd: String = rand::thread_rng().gen_ascii_chars().take(8).collect();
                    let trip = trip::trip(&passwd);

                    if trip.chars()
                        .next()
                        .and_then(|c| if c == '#' {
                            None
                        } else {
                            env::args().find(|a| trip.contains(a.as_str()))
                        })
                        .is_some()
                    {
                        println!("#{} => {}", passwd, trip);
                    }

                    count += 1;
                }

                count
            })
        })
        .collect();

    io::stdin().bytes().next().unwrap().unwrap();
    abort.store(true, Ordering::Relaxed);
    let count: u64 = threads.into_iter().map(|t| t.join().unwrap()).sum();
    let count_per_second = count / now.elapsed().as_secs();
    println!(
        "Processed {} tripcodes ({}/second)",
        count,
        count_per_second
    );
}
