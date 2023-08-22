use anyhow::Result;
use itertools::Itertools;
use num_format::{Locale,  ToFormattedString};
use std::fs::File;
use std::io::{BufRead, BufReader, LineWriter, Write};
use std::sync::{Arc, Mutex, atomic::{AtomicUsize, Ordering}};
use std::time::{SystemTime, Duration};
use std::thread;
use std::vec::Vec;

fn print_status(txt: &str, count:usize, elapsed: Duration) {
    println!("{:>12} {} ({} ms)", count.to_formatted_string(&Locale::en), txt,
        elapsed.as_millis().to_formatted_string(&Locale::en));
}

fn main() ->Result<()>{
    let start = SystemTime::now();
    println!("BEGIN");
    let now = SystemTime::now();
    let all_words = read_file("./words_alpha.txt")?;
    print_status("Words read from input file", all_words.len(), now.elapsed()?);
    let now = SystemTime::now();
    let five_words = trim_words(all_words);
    print_status("Found 5 letter words", five_words.len(), now.elapsed()?);
    let now = SystemTime::now();
    let word_masks = calculate_masks(&five_words);
    print_status("Calculated letter masks", word_masks.len(), now.elapsed()?);
    let now = SystemTime::now();
    let uniq_masks = dedup_masks(&word_masks);
    print_status("Unique letter masks", uniq_masks.len(), now.elapsed()?);
    let now = SystemTime::now();
    let solution_masks = wordle_bitmasks(uniq_masks)?;
    print_status("Solution maps found", solution_masks.len(), now.elapsed()?);
    let now = SystemTime::now();
    let solutions = map_solutions(solution_masks, five_words, word_masks.into_iter().map(|(m,_)| m).collect_vec());
    print_status("Total solutions found", solutions.len(), now.elapsed()?);
    let now = SystemTime::now();
    write_lines(&solutions, "wordle.txt")?;
    print_status("Solutions written to file", solutions.len(), now.elapsed()?);
    println!("END ({} ms)", start.elapsed()?.as_millis().to_formatted_string(&Locale::en));
    Ok(())
}

fn read_file(filepath: &str) -> Result<Vec<String>> {
    Ok(BufReader::new(File::open(filepath)?).lines().try_collect()?)
}

fn trim_words(words: Vec<String>) -> Vec<String> {
    words.iter().map(|w| String::from(w.trim().to_lowercase())).filter(|w| w.len() == 5).sorted().collect_vec()
}

fn calculate_masks(words: &Vec<String>) -> Vec<(u64, bool)> {
    words.iter().map(|word| {
        let mut is_valid = true;
        let mut bitmask = 0u64;
        for c in word.bytes() {
            let i = c as i64 - b'a' as i64;
            assert!(i >= 0 && i < 26);
            is_valid &= (bitmask & 1 << i) == 0;
            bitmask |= 1 << i;
        }
        (bitmask, is_valid)
    }).collect_vec()
}

fn dedup_masks(masks: &Vec<(u64,bool)>) -> Vec<u64> {
    masks.iter().filter_map(|(mask, is_valid)| is_valid.then_some(*mask)).sorted().dedup().collect_vec()
}

fn threadded_loop(bitmasks: Vec<u64>, i: Arc<AtomicUsize>, result: Arc<Mutex<Vec<[u64;5]>>>) {
    loop {
        let i = i.fetch_add(1, Ordering::SeqCst);
        if i >= bitmasks.len() { break; }
        let key_a = bitmasks[i];
        let mask = key_a;

        let filter = |key: u64, masks: &[u64] | masks.iter().copied().filter(|m| *m & key == 0).collect_vec();

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
                    if masks_e.len() == 0 { continue }
                    let res = masks_e.iter().map(|&key_e| [key_a, key_b, key_c, key_d, key_e]);
                    let mut result = result.lock().unwrap();
                    result.extend(res);
                }
            }
        }
    }
}

fn wordle_bitmasks(bitmasks: Vec<u64>) -> Result<Vec<[u64;5]>> {
    let result: Vec<[u64;5]> = Vec::new();
    let mutex = Arc::new(Mutex::new(result));
    let i = Arc::new(AtomicUsize::new(0));
    let mut threads = Vec::new();
    for _ in 0..thread::available_parallelism()?.get() {
        let (mutex, bitmasks, i) = (mutex.clone(), bitmasks.clone(), i.clone());
        threads.push(thread::spawn(move || threadded_loop(bitmasks, i, mutex)));
    }
    threads.into_iter().for_each(|t| t.join().unwrap());
    let result = (*mutex.lock().unwrap()).to_vec();
    Ok(result)
}

fn map_solutions(solutions: Vec<[u64;5]>, words: Vec<String>, bitmasks: Vec<u64>) -> Vec<String> {
    solutions.into_iter()
        .flat_map(|solution| {
            solution.into_iter()
            .map(|mask| words.iter().zip(bitmasks.iter()).filter_map(move |(w,m)| (mask == *m).then_some(w)))
            .multi_cartesian_product()
            .map(|solution| solution.into_iter().sorted().fold(String::from(""), |acc, i| format!("{} {}", acc, i)))
        })
        .sorted().collect_vec()
}

fn write_lines(lines:&Vec<String>, filepath: &str) -> Result<()> {
    let mut file = LineWriter::new(File::create(filepath)?);
    for line in lines {
        file.write(format!("{}\n", line).as_bytes())?;
    }
    Ok(())
}