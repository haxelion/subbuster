use std::os;
use std::io::File;
use std::io::BufferedReader;
use std::num::Float;
use std::str;
use std::iter::IteratorExt;

struct Probabilistic<T> {
    p : f64,
    v : T,
}

struct Dictionary {
    words : Vec<Vec<u8>>,
    unigram : [f64, ..256],
    sort_order : [uint, ..256] 
}

impl Dictionary {
    fn new() -> Dictionary {
        Dictionary { words: Vec::new(),
                     unigram : [0f64, ..256],
                     sort_order : [0u, ..256]}
    }
}

fn print_usage() {
    println!("subbuster input output");
}

fn main() {
    let mut args: Vec<String> = os::args();
    let mut dict = Dictionary::new();
    let mut lenght: Vec<Probabilistic<uint>> = Vec::new();
    let mut best_score : f64;
    let mut best_key : Vec<[u8, ..2]> = Vec::new();
    let mut i : uint;

    if args.len() < 3 {
        print_usage();
        return;
    }
    let dictionary_path = args.pop().unwrap();
    let input = args.pop().unwrap();
    i = 0;
    while i+1 < args.len() {
        if args[i].as_slice() == "-l" {
            i += 1;
            match str::from_str::<uint>(args[i].as_slice()) {
                Some(l) => {
                    lenght.push(Probabilistic {p : 1f64, v : l});
                },
                None => {
                    println!("{} is not a valid key lenght", args[i]);
                    print_usage();
                    return;
                }
            }
        }
        i += 1;
    }

    read_dictionary(dictionary_path.as_slice(), &mut dict);
    let data  = match File::open(&Path::new(input.as_slice())).read_to_end() {
        Ok(d) => { d },
        Err(e) => {println!("Could not read input file: {}", e); return;}
    };

    if lenght.is_empty() {
        find_lenght_candidates(data.as_slice(), &mut lenght, 10);
    }

    best_score = 1f64;
    lenght.truncate(5);
    for l in lenght.iter() {
        let mut key : Vec<[u8, ..2]> = Vec::new();
        let score = fast_adapt_lvl2(data.as_slice(), &dict, l.v, &mut key);
        if score < best_score {
            best_key = key.clone();
            best_score = score;
        }
    }
    print!("{} : x = ", 1.0 - best_score);
    for i in range(0, best_key.len()) {
        print!("{:02x}", best_key[i][0]);
    }
    print!(" a = ");
    for i in range(0, best_key.len()) {
        print!("{:02x}", best_key[i][1]);
    }
    print!("\n");
}

fn find_lenght_candidates(data : &[u8], lenght : &mut Vec<Probabilistic<uint>>, max_l : uint) {
    for l in range(1, max_l+1) {
        lenght.push(Probabilistic{ p : 0f64, v : l});
        for p in range(0, l) {
            let mut freq = [0u64, ..256];
            let mut sum = 0u64;
            let mut var = 0f64;
            let mut i = p;
            while i < data.len() {
                freq[data[i] as uint] += 1u64;
                sum += 1u64;
                i += l;
            }
            for i in range(0u, 256u) {
                let diff = (freq[i] as f64 / sum as f64)-(1f64/256f64);
                var += diff*diff;
            }
            lenght[l-1].p += var / (l as f64).powf(1.5);
        }
    }
    lenght.sort_by(|a, b| {
        if b.p < a.p { Less }
        else if b.p > a.p { Greater }
        else { Equal }
    });
}

fn read_dictionary(path : &str, dict : &mut Dictionary)
{
    let file  = match File::open(&Path::new(path.as_slice())) {
        Ok(f) => { f },
        Err(e) => {println!("Could not read dictionary file: {}", e); return;}
    };
    let mut reader = BufferedReader::new(file);
    let mut freq = [0u64, ..256];
    let mut sum = 0u64;
    loop {
        let line = match reader.read_line() {
            Ok(l) => { l.into_bytes() },
            Err(_) => { break; }
        };
        dict.words.push(line.clone());
        for c in line.iter() {
            sum += 1;
            freq[*c as uint] += 1;
        }
    }   
    for i in range(0u, 256) {
        dict.unigram[i] = freq[i] as f64 / sum as f64;
    }
}

fn compute_unigram_var(u1 : &[f64, ..256], u2 : &[f64, ..256], s : &[uint, ..256]) -> f64 {
    let mut cost : f64 = 0f64;
    for i in range(0u, 256) {
        let c = u1[i] - u2[s[i]];
        cost += c*c;
    }
    return cost;
}

fn gen_lvl1_sub(x : u8, sub : &mut [uint, ..256]) {
    for i in range(0u, 256) {
        sub[i] = i ^ x as uint;
    }
}

fn gen_lvl2_sub(x : u8, a : u8, sub : &mut [uint, ..256]) {
    for i in range(0u, 256) {
        sub[i] = ((i as u8 ^ x) + a) as uint;
    }
}


fn fast_adapt_lvl1(data : &[u8], dict : &Dictionary, l : uint, key : &mut Vec<u8>) -> f64 {
    let mut unigram : Vec<[f64, ..256]> = Vec::from_fn(l, |_| [0f64, ..256]);
    let mut score : Vec<f64> = Vec::from_elem(l, 1f64);
    key.clear();
    for p in range(0u, l) {
        let mut i = p;
        let mut freq = [0u64, ..256];
        let mut sub = [0u, ..256];
        let mut sum = 0u64;
        while i < data.len() {
            sum += 1;
            freq[data[i] as uint] += 1; 
            i += l;
        }
        for i in range(0u, 256) {
            unigram[p][i] = freq[i] as f64 / sum as f64;
        }
        key.push(0);
        for k in range(0u, 256) {
            gen_lvl1_sub(k as u8, &mut sub);
            let s = compute_unigram_var(&dict.unigram, &unigram[p], &sub);
            if s < score[p] {
                score[p] = s;
                key[p] = k as u8;
            }
        }
    }
    return score.iter().fold(0f64, |a, &v| a + v);
}

fn fast_adapt_lvl2(data : &[u8], dict : &Dictionary, l : uint, key : &mut Vec<[u8, ..2]>) -> f64 {
    let mut unigram : Vec<[f64, ..256]> = Vec::from_fn(l, |i| [0f64, ..256]);
    let mut score : Vec<f64> = Vec::from_elem(l, 1f64);
    key.clear();
    for p in range(0u, l) {
        let mut i = p;
        let mut freq = [0u64, ..256];
        let mut sub = [0u, ..256];
        let mut sum = 0u64;
        while i < data.len() {
            sum += 1;
            freq[data[i] as uint] += 1; 
            i += l;
        }
        for i in range(0u, 256) {
            unigram[p][i] = freq[i] as f64 / sum as f64;
        }
        key.push([0, 0]);
        for x in range(0u, 256) {
            for a in range(0u, 256) {
                gen_lvl2_sub(x as u8, a as u8, &mut sub);
                let s = compute_unigram_var(&dict.unigram, &unigram[p], &sub);
                if s < score[p] {
                    score[p] = s;
                    key[p][0] = x as u8;
                    key[p][1] = a as u8;
                }
            }
        }
    }
    return score.iter().fold(0f64, |a, &v| a + v);
}
