use std::os;
use std::io::File;
use std::io::BufferedReader;
use std::num::Float;
use std::iter::IteratorExt;

struct Probabilistic<T> {
    p : f64,
    v : T,
}

struct Dictionary {
    words : Vec<Vec<u8>>,
    unigram : [f64, ..256]
}

impl Dictionary {
    fn new() -> Dictionary {
        Dictionary {
            words: Vec::new(),
            unigram : [0f64, ..256]
        }
    }
}

fn print_usage() {
    println!("subbuster input output");
}

fn main() {
    let mut args: Vec<String> = os::args();
    let mut dict = Dictionary::new();
    let mut lenght: Vec<Probabilistic<uint>> = Vec::new();
    let mut verbose = false;
    let mut model = 1u;
    let mut i : uint;

    if args.len() < 3 {
        print_usage();
        return;
    }
    let dictionary_path = args.pop().unwrap();
    let input = args.pop().unwrap();
    i = 0;
    while i < args.len() {
        if args[i].as_slice() == "-v" {
            verbose = true;
        }
        else if args[i].as_slice() == "-m" {
            i += 1;
            if i >= args.len() {
                println!("No model number given");
                print_usage();
                return;
            }
            model = match from_str::<uint>(args[i].as_slice()) {
                Some(1) => 1,
                Some(2) => 2,
                _ => {
                    println!("{} is not a valid model number", args[i]);
                    print_usage();
                    return;
                }
            };
        }
        else if args[i].as_slice() == "-l" {
            i += 1;
            if i >= args.len() {
                println!("No key lenght given");
                print_usage();
                return;
            }
            match from_str::<uint>(args[i].as_slice()) {
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
        if verbose {
            println!("Lenght candidates: ");
            println!("------------------\n");
            println!("P        | l");
            for l in lenght.iter() {
                println!("{:.6} : {}", l.p, l.v);
            }
            print!("\n\n");
        }
    }

    let mut best_score = 0f64;
    let mut best_key : Vec<Vec<u8>> = Vec::new();
    lenght.truncate(5);
    if verbose {
        println!("Key candidates for model {}:", model);
        println!("----------------------------\n");
        println!("P        | l   | K");
    }
    for l in lenght.iter() {
        let mut key : Vec<Vec<u8>> = Vec::new();
        let score = match model {
            1 => fast_adapt_lvl1(data.as_slice(), &dict, l.v, &mut key),
            2 => fast_adapt_lvl2(data.as_slice(), &dict, l.v, &mut key),
            _ => {return;}
        };
        if score > best_score {
            best_key = key.clone();
            best_score = score;
        }
        if verbose {
            print!("{:.6} : {:3} : x = ", score, l.v);
            for b in key[0].iter() {
                print!("{:02x}", *b);
            }
            if key.len() > 1 {
                print!(" a = ");
                for b in key[1].iter() {
                    print!("{:02x}", *b);
                }
            }
            print!("\n");
        }
    }
    print!("Best key: {:.6} : {:3} : x = ", best_score, best_key[0].len());
    for b in best_key[0].iter() {
        print!("{:02x}", *b);
    }
    if best_key.len() > 1 {
        print!(" a = ");
        for b in best_key[1].iter() {
            print!("{:02x}", *b);
        }
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


fn fast_adapt_lvl1(data : &[u8], dict : &Dictionary, l : uint, key : &mut Vec<Vec<u8>>) -> f64 {
    let mut unigram : Vec<[f64, ..256]> = Vec::from_fn(l, |_| [0f64, ..256]);
    let mut score : Vec<f64> = Vec::from_elem(l, 1f64);
    key.clear();
    key.push(Vec::from_elem(l, 0u8));
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
        for k in range(0u, 256) {
            gen_lvl1_sub(k as u8, &mut sub);
            let s = compute_unigram_var(&dict.unigram, &unigram[p], &sub);
            if s < score[p] {
                score[p] = s;
                key[0][p] = k as u8;
            }
        }
    }
    return score.iter().fold(1f64, |a, &v| a - v*10f64/(l as f64).powf(1.5));
}

fn fast_adapt_lvl2(data : &[u8], dict : &Dictionary, l : uint, key : &mut Vec<Vec<u8>>) -> f64 {
    let mut unigram : Vec<[f64, ..256]> = Vec::from_fn(l, |_| [0f64, ..256]);
    let mut score : Vec<f64> = Vec::from_elem(l, 1f64);
    key.clear();
    key.push(Vec::from_elem(l, 0u8));
    key.push(Vec::from_elem(l, 0u8));
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
        for x in range(0u, 256) {
            for a in range(0u, 256) {
                gen_lvl2_sub(x as u8, a as u8, &mut sub);
                let s = compute_unigram_var(&dict.unigram, &unigram[p], &sub);
                if s < score[p] {
                    score[p] = s;
                    key[0][p] = x as u8;
                    key[1][p] = a as u8;
                }
            }
        }
    }
    return score.iter().fold(1f64, |a, &v| a - v*10f64/(l as f64).powf(1.5));
}
