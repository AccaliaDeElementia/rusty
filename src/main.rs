use std::fs::File;
use std::io::{self, BufRead, LineWriter, Write};
use std::path::Path;
use std::collections::HashMap;
use std::vec::Vec;
use std::time::SystemTime;
use std::thread;
use std::sync::{Arc, Mutex};

fn main() {
    let now = SystemTime::now();
    let filepath = "./words_alpha.txt";
    println!("BEGIN");
    let words = read_vec(filepath);
    let elapsed = now.elapsed();
    println!("{} valid wordle words", words.len());
    println!("{} ms to read file to map", elapsed.expect("should be ok").as_millis());
    let now = SystemTime::now();
    let wordles = wordle(words);
    let elapsed = now.elapsed();
    println!("{} valid wordle combos", wordles.len());
    println!("{} ms to wordle", elapsed.expect("should be ok").as_millis());
    let file = File::create("wordle.txt").expect("Error creating output file!");
    let mut file = LineWriter::new(file);
    for wordle in wordles {
        file.write(format!("{}\n", wordle).as_bytes()).expect("Error writing line to output file!");
    }
    println!("END");
}

fn filter_words(key: u64, words: &[(u64, Vec<String>)]) -> Vec<(u64, Vec<String>)> {
    let mut filtered: Vec<(u64, Vec<String>)> = Vec::new();
    for (k, v) in words {
        if k & key == 0 {
            filtered.push((*k, v.to_vec()));
        }
    }
    filtered
}

fn append_words (a: &Vec<String>, b: &Vec<String>) -> Vec<String> {
    let mut result: Vec<String> = Vec::new();
    for a in a {
        for b in b {
            result.push(format!("{} {}", a, b));
        }
    }
    result
}

fn wordle (words: Vec<(u64, Vec<String>)>) -> Vec<String> {
    let now = SystemTime::now();
    let result: Vec<String> = Vec::new();
    let mutex = Arc::new(Mutex::new(result));
    let i = Arc::new(Mutex::new(0));
    let mut threads = Vec::new();
    for _ in 0..thread::available_parallelism().expect("should be able to read core count").get() {
        let mutex = Arc::clone(&mutex);
        let words = words.clone();
        let i = i.clone();
        let now = now.clone();
        threads.push(thread::spawn(move || {
            let mut a;
            loop {
                {
                    let mut i = i.lock().unwrap();
                    a = *i;
                    *i += 1;
                }
                if a >= words.len() { break; }
                let (key, aword) = &words[a];
                if a % 100 == 0 || a == words.len() - 1 {
                    let elapsed = now.elapsed().expect("TIME ERROR");
                    let seconds = elapsed.as_secs() % 60;
                    let minutes = (elapsed.as_secs() / 60) % 60;
                    let hours = (elapsed.as_secs() / 60) / 60;
                    let percent = 100.0 * ( (a + 1) as f64 / words.len() as f64);
                    println!("{} - {hours:0>2}:{minutes:0>2}:{seconds:0>2} - {percent:8.4}%", &aword[0]);
                }
                let innerwords = filter_words(*key, &words[(a+1)..]);
                for (b, (bkey, bword)) in innerwords.iter().enumerate() {
                    let key = key | bkey;
                    let innerwords = filter_words(key, &innerwords[(b+1)..]);
                    for (c, (ckey, cword)) in innerwords.iter().enumerate() {
                        let key = key | ckey;
                        let innerwords = filter_words(key, &innerwords[(c+1)..]);
                        for (d, (dkey, dword)) in innerwords.iter().enumerate() {
                            let key = key | dkey;
                            let innerwords = filter_words(key, &innerwords[(d+1)..]);
                            for (_, (_, eword)) in innerwords.iter().enumerate() {
                                let mut vec = mutex.lock().unwrap();
                                vec.extend(append_words(aword, &append_words(bword, &append_words(cword, &append_words(dword, eword)))));
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

    let x = (*mutex.lock().unwrap()).to_vec();
    x
}

fn read_vec<P> (filepath: P) -> Vec<(u64, Vec<String>)>
where P: AsRef<Path>, {
    let mut words: Vec<(u64, Vec<String>)> = Vec::new();
    let mut map: HashMap<u64, Vec<String>> = HashMap::new();

    if let Ok(lines) = read_lines(filepath) {
        for line in lines {
            if let Ok(word) = line {
                let (valid, bits) = string_to_bitfield(&word);
                if !valid { continue }
                if let Some(bin) = map.get_mut(&bits) {
                    bin.push(word);
                } else {
                    let mut bin = Vec::new();
                    bin.push(word);
                    map.insert(bits, bin);
                }
            }
        }
    }
    for (bits, vec) in map.iter() {
        words.push((*bits,vec.to_vec()));
    }
    words.sort_by(|(_, a), (_, b) | a[0].cmp(&b[0]));
    words

}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn char_to_bit(s: char) -> u64 {
    match s {
        'a' => 1,
        'b' => 1<<1,
        'c' => 1<<2,
        'd' => 1<<3,
        'e' => 1<<4,
        'f' => 1<<5,
        'g' => 1<<6,
        'h' => 1<<7,
        'i' => 1<<8,
        'j' => 1<<9,
        'k' => 1<<10,
        'l' => 1<<11,
        'm' => 1<<12,
        'n' => 1<<13,
        'o' => 1<<14,
        'p' => 1<<15,
        'q' => 1<<16,
        'r' => 1<<17,
        's' => 1<<18,
        't' => 1<<19,
        'u' => 1<<20,
        'v' => 1<<21,
        'w' => 1<<22,
        'x' => 1<<23,
        'y' => 1<<24,
        'z' => 1<<25,
        _ => 1<<26
    }
}

fn string_to_bitfield(s: &String) -> (bool, u64) {
    let s = s.to_lowercase();
    let mut count = 0;
    let mut result = 0;
    for (_i, letter) in s.chars().enumerate() {
        let bit = char_to_bit(letter);
        if result & bit == 0 {
            count = count + 1;
        }
        result = result | char_to_bit(letter);
    }
    let valid_wordle = s.len() == 5 && count == 5;
    (valid_wordle, result)
}