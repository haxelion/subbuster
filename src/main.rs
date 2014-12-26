/*
This file is part of SubBuster.

SubBuster is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

SubBuster is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <http://www.gnu.org/licenses/>.

Copyright 2014 Charles Hubain <github@haxelion.eu>
*/

use std::os;
use std::io::File;
use std::num::Float;
use std::iter::IteratorExt;
use std::iter::range_step;
use std::thread::Thread;

struct Probabilistic<T> {
    p : f64,
    v : T,
}

struct Sample {
    data : Vec<u8>,
    unigram : [f64, ..256]
}

impl Sample {
    fn new() -> Sample {
        Sample {
            data: Vec::new(),
            unigram : [0f64, ..256]
        }
    }
}

enum Model {Level1, Level2, Level3}

struct SBTask {
    x : u8,
    a : u8,
    m : u16,
    p : uint,
    score : f64
}

fn print_usage() {
    println!("subbuster [-m [1|2|3]] [-l l] [-v] input sample");
    println!("");
    println!("* input: input file to decipher.");
    println!("* sample: some plaintext sample from which byte the frequency distribution is ");
    println!("computed.");
    println!("* -m: optional model level number, default to 1. Model level 1 is xor, model ");
    println!("level 2 is xor-add, model level 3 is xor-add-mix.");
    println!("* -l: optional key lenght. If not provided, subbuster attempts to guess the key ");
    println!("lenght using entropy.");
    println!("* -v: verbose mode, the results from all the candidates");
    println!("");
    println!("Warning: model level 3 is really slow (a few hours on a modern computer) ");
    println!("because it attempts to bruteforce all 2 642 411 520 key possibilites per byte. ");
    println!("A faster algorithm is planned.");
}

fn main() {
    let mut args: Vec<String> = os::args();
    let mut sample = Sample::new();
    let mut lenght: Vec<Probabilistic<uint>> = Vec::new();
    let mut verbose = false;
    let mut model = Model::Level1;
    let mut i : uint;

    if args.len() < 3 {
        print_usage();
        return;
    }
    let sample_path = args.pop().unwrap();
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
            model = match args[i].as_slice().parse() {
                Some(1u) => Model::Level1,
                Some(2u) => Model::Level2,
                Some(3u) => Model::Level3,
                _ => {
                    println!("{} is not a valid model level", args[i]);
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
            match args[i].as_slice().parse() {
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

    read_sample(sample_path.as_slice(), &mut sample);
    let data  = match File::open(&Path::new(input.as_slice())).read_to_end() {
        Ok(d) => { d },
        Err(e) => {println!("Could not read input file: {}", e); return;}
    };

    if lenght.is_empty() {
        find_lenght_candidates(data.as_slice(), &mut lenght, 10);
        if verbose {
            println!("Lenght candidates: ");
            println!("------------------\n");
            println!("S        | l");
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
        println!("Key candidates:");
        println!("---------------\n");
        println!("S        | l   | K");
    }
    for l in lenght.iter() {
        let mut key : Vec<Vec<u8>> = Vec::new();
        let score = match model {
            Model::Level1 => break_lvl1(data.as_slice(), &sample, l.v, &mut key),
            Model::Level2 => break_lvl2(data.as_slice(), &sample, l.v, &mut key),
            Model::Level3 => break_lvl3(data.as_slice(), &sample, l.v, &mut key),
        };
        if score > best_score {
            best_key = key.clone();
            best_score = score;
        }
        if verbose {
            print!("{:.6} : {:3} : ", score, l.v);
            print_key(&key);
            print!("\n");
        }
    }

    if verbose {
        print!("\n");
    }

    print!("Best key: {:.6} : {:3} : ", best_score, best_key[0].len());
    print_key(&best_key);
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
            lenght[l-1].p += 10f64 * var / (l as f64).powf(1.2);
        }
    }
    lenght.sort_by(|a, b| {
        if b.p < a.p { Less }
        else if b.p > a.p { Greater }
        else { Equal }
    });
}

fn read_sample(path : &str, sample : &mut Sample)
{
    sample.data = match File::open(&Path::new(path.as_slice())).read_to_end() {
        Ok(d) => { d },
        Err(e) => {
            println!("Could not read sample file: {}", e);
            panic!();
        }
    };
    let mut freq = [0u64, ..256];
    let mut sum = 0u64;
    for c in sample.data.iter() {
        sum += 1;
        freq[*c as uint] += 1;
    }
    for i in range(0u, 256) {
        sample.unigram[i] = freq[i] as f64 / sum as f64;
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

fn gen_lvl3_sub(x : u8, a : u8, m : u16, sub : &mut [uint, ..256]) {
    let c = [40320u16, 5040u16, 720u16, 120u16, 24u16, 6u16, 2u16, 1u16, 1u16];
    let mut used = [false, ..8];
    let mut p = [0u, ..8];
    for i in range(0u, 8u) {
        p[i] = ((m%c[i])/c[i+1]+1) as uint;
        for j in range(0u, 8) {
            if used[j] == false {
                p[i] -= 1u;
            }
            if p[i] == 0 {
                p[i] = j;
                used[j] = true;
                break;
            }
        }
    }
    for i in range(0u, 256) {
        let b = (i as u8 ^ x) + a;
        sub[i] = ((b & 1u8) << p[0] |
                 ((b & 2u8) >> 1u) << p[1] |
                 ((b & 4u8) >> 2u) << p[2] |
                 ((b & 8u8) >> 3u) << p[3] |
                 ((b & 16u8) >> 4u) << p[4] |
                 ((b & 32u8) >> 5u) << p[5] |
                 ((b & 64u8) >> 6u) << p[6] |
                 ((b & 128u8) >> 7u) << p[7]) as uint;
    }
}


fn break_lvl1(data : &[u8], sample : &Sample, l : uint, key : &mut Vec<Vec<u8>>) -> f64 {
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
            let s = compute_unigram_var(&sample.unigram, &unigram[p], &sub);
            if s < score[p] {
                score[p] = s;
                key[0][p] = k as u8;
            }
        }
    }
    return score.iter().fold(1f64, |a, &v| a - 10f64 * v / l as f64);
}

fn break_lvl2(data : &[u8], sample : &Sample, l : uint, key : &mut Vec<Vec<u8>>) -> f64 {
    let mut unigram : Vec<[f64, ..256]> = Vec::from_fn(l, |_| [0f64, ..256]);
    let mut score : Vec<f64> = Vec::from_elem(l, 1f64);
    key.clear();
    key.push(Vec::from_elem(l, 0u8));
    key.push(Vec::from_elem(l, 0u8));
    for p in range(0u, l) {
        let mut i = p;
        let mut freq = [0u64, ..256];
        let mut sum = 0u64;
        while i < data.len() {
            sum += 1;
            freq[data[i] as uint] += 1; 
            i += l;
        }
        for i in range(0u, 256) {
            unigram[p][i] = freq[i] as f64 / sum as f64;
        }
    }
    let (tx, rx) = channel::<SBTask>();
    for p in range(0u, l) {
        let tx = tx.clone();
        let (u_tx, u_rx) = channel::<[f64, ..256]>();
        u_tx.send(sample.unigram);
        u_tx.send(unigram[p]);
        Thread::spawn(move || {
            let mut sub = [0u, ..256];
            let mut res = SBTask {x : 0u8, a : 0u8, m : 0u16, p : p, score : 1f64};
            let du = u_rx.recv();
            let u = u_rx.recv();
            for x in range(0u, 256) {
                for a in range(0u, 256) {
                    gen_lvl2_sub(x as u8, a as u8, &mut sub);
                    let s = compute_unigram_var(&du, &u, &sub);
                    if s < res.score {
                        res.score = s;
                        res.x = x as u8;
                        res.a = a as u8;
                    }
                }
            }
            tx.send(res);
        }).detach();
    }
    for p in range(0u, l) {
        let res = rx.recv();
        score[res.p] = res.score;
        key[0][res.p] = res.x as u8;
        key[1][res.p] = res.a as u8;
    }
    return score.iter().fold(1f64, |a, &v| a - 10f64 * v / l as f64);
}

fn break_lvl3(data : &[u8], sample : &Sample, l : uint, key : &mut Vec<Vec<u8>>) -> f64 {
    let mut unigram : Vec<[f64, ..256]> = Vec::from_fn(l, |_| [0f64, ..256]);
    let mut score : Vec<f64> = Vec::from_elem(l, 1f64);
    key.clear();
    key.push(Vec::from_elem(l, 0u8));
    key.push(Vec::from_elem(l, 0u8));
    key.push(Vec::from_elem(2*l, 0u8));
    for p in range(0u, l) {
        let mut i = p;
        let mut freq = [0u64, ..256];
        let mut sum = 0u64;
        while i < data.len() {
            sum += 1;
            freq[data[i] as uint] += 1; 
            i += l;
        }
        for i in range(0u, 256) {
            unigram[p][i] = freq[i] as f64 / sum as f64;
        }
        let n_cpus = os::num_cpus();
        let (tx, rx) = channel::<SBTask>();
        for i in range(0u, n_cpus) {
            let tx = tx.clone();
            let (u_tx, u_rx) = channel::<[f64, ..256]>();
            u_tx.send(sample.unigram);
            u_tx.send(unigram[p]);
            Thread::spawn(move || {
                let mut sub = [0u, ..256];
                let mut res = SBTask {x : 0u8, a : 0u8, m : 0u16, p : p,score : 1f64};
                let du = u_rx.recv();
                let u = u_rx.recv();
                for x in range_step(i, 256, n_cpus) {
                    for a in range(0u, 256) {
                        for m in range(0u, 40320) {
                            gen_lvl3_sub(x as u8, a as u8, m as u16, &mut sub);
                            let s = compute_unigram_var(&du, &u, &sub);
                            if s < res.score {
                                res.score = s;
                                res.x = x as u8;
                                res.a = a as u8;
                                res.m = m as u16;
                            }
                        }
                    }
                }
                tx.send(res);
            }).detach();
        }
        for i in range(0u, n_cpus) {
            let res = rx.recv();
            if res.score < score[p] {
                score[p] = res.score;
                key[0][p] = res.x as u8;
                key[1][p] = res.a as u8;
                key[2][2*p] = (res.m >> 8) as u8;
                key[2][2*p+1] = (res.m & 0xff) as u8;
            }
        }
    }
    return score.iter().fold(1f64, |a, &v| a - 10f64 * v / l as f64);
}

fn print_key(key : &Vec<Vec<u8>>) {
    print!("x = ");
    for b in key[0].iter() {
        print!("{:02x}", *b);
    }
    if key.len() > 1 {
        print!(" a = ");
        for b in key[1].iter() {
            print!("{:02x}", *b);
        }
    }
    if key.len() > 2 {
        print!(" m = ");
        for b in key[2].iter() {
            print!("{:02x}", *b);
        }
    }
}
