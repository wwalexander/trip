#![feature(test)]

extern crate rand;
extern crate test;

use rand::{Rng, NewRng, XorShiftRng};
use std::env;
use std::io::{self, Read};
use std::str;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Instant;

mod trip;

const SAMPLES: [u8; 64] = *b"./0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

#[inline(always)]
fn rand_passwd<R: Rng>(rng: &mut R) -> [u8; 8] {
    let var: usize = rng.gen();

    [
	SAMPLES[ var        & 0x3f],
	SAMPLES[(var >>  8) & 0x3f],
	SAMPLES[(var >> 16) & 0x3f],
	SAMPLES[(var >> 24) & 0x3f],
	SAMPLES[(var >> 32) & 0x3f],
	SAMPLES[(var >> 40) & 0x3f],
	SAMPLES[(var >> 48) & 0x3f],
	SAMPLES[(var >> 56) & 0x3f],
    ]
}

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
		let mut rng = XorShiftRng::new();
		let mut count = 0;

		while !abort.load(Ordering::Relaxed) {
		    let passwd = rand_passwd(&mut rng);
		    let tripcode = trip::trip(passwd);
		    let tripcode_str = str::from_utf8(&tripcode).unwrap();

		    if env::args().find(|a| tripcode_str.contains(a.as_str())).is_some() {
		        println!("#{} => {}", str::from_utf8(&passwd).unwrap(), tripcode_str);
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
    println!("Processed {} tripcodes ({}/second)", count, count_per_second);
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    #[bench]
    fn bench_rand_passwd(b: &mut Bencher) {
	let mut rng = XorShiftRng::new();
	b.iter(|| rand_passwd(&mut rng));
    }
}
