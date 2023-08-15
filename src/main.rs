use std::fs::File;
use std::io::{self, BufRead, LineWriter, Write};
use std::path::Path;
use std::collections::HashMap;
use std::vec::Vec;
use std::time::SystemTime;
use std::thread;
use std::sync::{Arc, Mutex, mpsc};

fn main() {
    let now = SystemTime::now();
    let filepath = "./words_alpha.txt";
    println!("BEGIN");
    let words = read_vec(filepath);
    let elapsed = now.elapsed();
    println!("{} valid wordle words", words.len());
    println!("{} ms to read file to map", elapsed.expect("should be ok").as_millis());
    let now = SystemTime::now();
    let wordles = wordle_multiproc(words);
    let elapsed = now.elapsed();
    println!("{} valid wordle combos", wordles.len());
    println!("{} ms to wordle", elapsed.expect("should be ok").as_millis());
    let file = File::create("wordle.txt").expect("Error creating output file!");
    let mut file = LineWriter::new(file);
    for wordle in wordles {
        file.write(wordle.as_bytes()).expect("Error writing line to output file!");
    }
    println!("END");
}

fn wordle_multiproc (words: Vec<(u64, Vec<String>)>) -> Vec<String> {
    let threadcount = 32;
    let result: Vec<String> = Vec::new();
    let mutex = Arc::new(Mutex::new(result));
    let (post, poll) = mpsc::channel();
    let mut threads = Vec::new();
    for i in 0..threadcount {
        let post = post.clone();
        let mutex = Arc::clone(&mutex);
        let threadcount = threadcount.clone();
        let words = words.clone();
        let i = i.clone();
        threads.push(thread::spawn(move || {
            let mut a = i;
            while a < words.len() {
                let (key, aword) = &words[a];
                for (b, (bkey, bword)) in words[(a+1)..].iter().enumerate() {
                    if key & bkey != 0 { continue }
                    let key = key | bkey;
                    for (c, (ckey, cword)) in words[(a+b+1)..].iter().enumerate() {
                        if key & ckey != 0 { continue }
                        let key = key | ckey;
                        for (d, (dkey, dword)) in words[(a+b+c+1)..].iter().enumerate() {
                            if key & dkey != 0 { continue }
                            let key = key | dkey;
                            for (_e, (ekey, eword)) in words[(a+b+c+d+1)..].iter().enumerate() {
                                if key & ekey != 0 { continue }
                                for a in aword {
                                    for b in bword {
                                        for c in cword {
                                            for d in dword {
                                                for e in eword {
                                                    let mut vec = mutex.lock().unwrap();
                                                    vec.push(format!("{} {} {} {} {}\n", a, b, c, d, e));
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                a = a + threadcount;
                post.send(aword[0].clone()).unwrap();
            }
        }));
    }
    let throbber = thread::spawn(move || {
        let now = SystemTime::now();
        let mut i = 0;
        let len = words.len();
        while i < len {
            let word: String = poll.recv().unwrap();
            i = i + 1;
            if i % 100 == 0 || i == len {
                let elapsed = now.elapsed().expect("TIME ERROR");
                let seconds = elapsed.as_secs() % 60;
                let minutes = (elapsed.as_secs() / 60) % 60;
                let hours = (elapsed.as_secs() / 60) / 60;
                let percent: f64 = ((i as f64) / words.len() as f64) * 100.0;
                println!("{word} - {hours:0>2}:{minutes:0>2}:{seconds:0>2} - {percent:8.4}%");
            }
        }
    });

    for thread in threads {
        thread.join().unwrap();
    }
    throbber.join().unwrap();

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
    (s.len() == 5 && count == 5, result)
}