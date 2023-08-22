use anyhow::Result;
use itertools::Itertools;
use std::fs::File;
use std::io::{BufRead, BufReader, LineWriter, Write};
use std::vec::Vec;
use std::time::{SystemTime, Duration};
use std::thread;
use std::sync::{Arc, Mutex};
use num_format::{Locale,ToFormattedString};

fn print_status(txt: &str, count:usize, elapsed: Duration) {
    println!("{:>12} {} ({} ms)", count.to_formatted_string(&Locale::en), txt,
        elapsed.as_millis().to_formatted_string(&Locale::en));
}

fn main() ->Result<()>{
    let start = SystemTime::now();
    println!("BEGIN");
    let now = SystemTime::now();
    let wordfile = BufReader::new(File::open("./words_alpha.txt")?);
    let all_words: Vec<String> = wordfile.lines().try_collect()?;
    print_status("Words read from input file", all_words.len(), now.elapsed()?);
    let now = SystemTime::now();
    let five_words = all_words
        .iter()
        .map(|w| w.trim())
        .filter(|w| w.len() == 5)
        .sorted()
        .collect_vec();
    print_status("Found 5 letter words", five_words.len(), now.elapsed()?);
    let now = SystemTime::now();
    let word_masks = five_words
        .iter()
        .map(|w| word_to_bitmask(w))
        .collect_vec();
    print_status("Calculated letter masks", word_masks.len(), now.elapsed()?);
    let now = SystemTime::now();
    let uniq_masks = word_masks
        .iter()
        .filter_map(|(mask, is_valid)| is_valid.then_some(*mask))
        .sorted()
        .dedup()
        .collect_vec();
    print_status("Unique letter masks", uniq_masks.len(), now.elapsed()?);
    let now = SystemTime::now();
    let solution_masks = wordle_bitmasks(uniq_masks);
    print_status("solution maps found", solution_masks.len(), now.elapsed()?);
    let now = SystemTime::now();
    let solutions = solution_masks
        .into_iter()
        .flat_map(|solution| {
            solution
            .into_iter()
            .map(|mask| {
                five_words
                    .iter()
                    .zip(word_masks.iter())
                    .filter_map(move |(w,(m,_))| (mask == *m).then_some(*w))
            })
            .multi_cartesian_product()
            .map(|solution| solution.into_iter().sorted().collect_vec())
        })
        .sorted();
    print_status("Total solutions found", solutions.len(), now.elapsed()?);
    let now = SystemTime::now();
    let file = File::create("wordle.txt")?;
    let mut file = LineWriter::new(file);
    let lines = solutions.len();
    for solution in solutions {
        let line = solution.into_iter().fold(String::from(""), |acc, i| format!("{} {}", acc, i));
        file.write(format!("{line}\n").as_bytes())?;
    }
    print_status("Solutions written to file", lines, now.elapsed()?);
    println!("END ({} ms)", start.elapsed()?.as_millis().to_formatted_string(&Locale::en));
    Ok(())
}

fn word_to_bitmask(word: &str) -> (u64, bool) {
    let mut is_valid = true;
    let mut bitmask = 0u64;
    for c in word.bytes() {
        let i = c as i64 - b'a' as i64;
        assert!(i >= 0 && i < 26);
        let m = 1 << i;
        is_valid &= (bitmask & m) == 0;
        bitmask |= m;
    }
    (bitmask, is_valid)
}

fn wordle_bitmasks(bitmasks: Vec<u64>) -> Vec<[u64;5]> {
    let result: Vec<[u64;5]> = Vec::new();
    let mutex = Arc::new(Mutex::new(result));
    let i = Arc::new(Mutex::new(0));
    let mut threads = Vec::new();
    for _ in 0..thread::available_parallelism().expect("should be able to read core count").get() {
        let mutex = Arc::clone(&mutex);
        let bitmasks = bitmasks.clone();
        let muti = i.clone();
        threads.push(thread::spawn(move || {
            let mut i;
            loop {
                {
                    let mut muti = muti.lock().unwrap();
                    i = *muti;
                    *muti += 1;
                }
                if i >= bitmasks.len() { break; }
                let key_a = bitmasks[i];
                let mask = key_a;

                let filter = |key: u64, masks: &[u64] | masks
                    .iter()
                    .copied()
                    .filter(|m| *m & key == 0)
                    .collect_vec();

                let masks_b = filter(mask, &bitmasks[i+1..]);
                for (b, &key_b) in masks_b.iter().enumerate() {
                    let mask = mask | key_b;
                    let masks_c = filter(mask, &masks_b[b+1..]);
                    for (c, &key_c) in masks_c.iter().enumerate() {
                        let mask = mask | key_c;
                        let masks_d = filter(mask, &masks_c[c+1..]);
                        for (d, &key_d) in masks_d.iter().enumerate() {
                            let mask = mask | key_d;
                            let masks_e = filter(mask, &masks_d[d+1..]);
                            for &key_e in masks_e.iter() {
                                let mut vec = mutex.lock().unwrap();
                                vec.push([key_a, key_b, key_c, key_d, key_e]);
                            }
                        }
                    }
                }
            }
        }));
    }

    for thread in threads {
        thread.join().unwrap();
    }

    let result = (*mutex.lock().unwrap()).to_vec();
    result
}