/*
This file is part of DummyCrypt.

DummyCrypt is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

DummyCrypt is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <http://www.gnu.org/licenses/>.

Copyright 2014 Charles Hubain <github@haxelion.eu>
*/
extern crate serialize;

use std::vec::Vec;
use std::iter::repeat;
use std::cmp::max;
use std::io::prelude::*;
use std::fs::File;
use std::env;
use serialize::hex::FromHex;

fn print_usage() {
    println!("dummycrypt (-e|-d) [-x X] [-a A] [-m M] input output");
    println!("");
    println!("* -e: specify encryption mode");
    println!("* -d: specify decryption mode");
    println!("* -x: optional xor hex string of bytes");
    println!("* -a: optional add hex string of bytes");
    println!("* -m: optional mix hex string of big endian 16 bits unsigned integer");
    println!("* input: input file name");
    println!("* output: output file name");
    println!("");
    println!("The hex strings are padded with zeroes to the same number of elements.");
    println!("");
    println!("The elements of M represent any of the 40320 possible bijective bit mix ");
    println!("operations, their encoding is described in the documentation.");
    println!("");
    println!("The cipher encryption algorithm for each byte b is  MIX(ADD(XOR(b,x),a),m)");
    println!("where x, a, m are elements taken from X, A and M respectively and wrap around ");
    println!("when the input is bigger than the key.");
    println!("");
    println!("Copyright 2015 Charles Hubain <github@haxelion.eu>");
}

enum Mode {Missing, Encrypt, Decrypt}

fn main() {
    let mut args : Vec<String> = env::args().collect();
    let mut mode : Mode = Mode::Missing;
    let mut key : Vec<Vec<u8>> = repeat(Vec::<u8>::new()).take(3).collect();
    let mut input : &str = "";
    let mut output : &str = "";
    let mut i = 1;
    while i < args.len() {
        match &args[i][..] {
            "-e" => {
                mode = Mode::Encrypt;
            },
            "-d" => {
                mode = Mode::Decrypt;
            },
            "-x" => {
                i += 1;
                if i < args.len() {
                    key[0] = match args[i][..].from_hex() {
                        Ok(h) => h,
                        Err(e) => {
                            println!("xor hex string is invalid: {}", e);
                            return;
                        }
                    };
                }
                else {
                    println!("You need to provide a xor hex string after -x");
                    print_usage();
                    return;
                }
            },
            "-a" => {
                i += 1;
                if i < args.len() {
                    key[1] = match args[i][..].from_hex() {
                        Ok(h) => h,
                        Err(e) => {
                            println!("add hex string is invalid: {}", e);
                            return;
                        }
                    };
                }
                else {
                    println!("You need to provide an add hex string after -a");
                    print_usage();
                    return;
                }
            },
            "-m" => {
                i += 1;
                if i < args.len() {
                    key[2] = match args[i][..].from_hex() {
                        Ok(h) => h,
                        Err(e) => {
                            println!("mix hex string is invalid: {}", e);
                            return;
                        }
                    };
                }
                else {
                    println!("You need to provide a mix hex string after -m");
                    print_usage();
                    return;
                }
            },
            arg => {
                if input == "" {
                    input = arg;
                }
                else if output == "" {
                    output = arg;
                }
                else {
                    println!("Unrecognized argument {}", arg);
                    print_usage();
                    return;
                }
            }
        }
        i += 1;
    }
    let length = max(key[0].len(), max(key[1].len(), key[2].len()/2));
    key[0].resize(length, 0u8);
    key[1].resize(length, 0u8);
    key[2].resize(length*2, 0u8);
    match mode {
        Mode::Encrypt => dummy_crypt_file(&input, &output, &key),
        Mode::Decrypt => dummy_decrypt_file(&input, &output, &key),
        Mode::Missing => print_usage(),
    };
}

fn gen_sub(x : u8, a : u8, m : u16, sub : &mut [usize; 256]) {
    let c = [40320u16, 5040, 720, 120, 24, 6, 2, 1, 1];
    let mut used = [false; 8];
    let mut p = [0usize; 8];
    for i in 0usize.. 8 {
        p[i] = ((m%c[i])/c[i+1]+1) as usize;
        for j in (0usize..8) {
            if used[j] == false {
                p[i] -= 1;
            }
            if p[i] == 0 {
                p[i] = j;
                used[j] = true;
                break;
            }
        }
    }
    for i in 0usize..256 {
        let b = (i as u8 ^ x) + a;
        sub[i] = ((b & 1) << p[0] |
                 ((b & 2) >> 1) << p[1] |
                 ((b & 4) >> 2) << p[2] |
                 ((b & 8) >> 3) << p[3] |
                 ((b & 16) >> 4) << p[4] |
                 ((b & 32) >> 5) << p[5] |
                 ((b & 64) >> 6) << p[6] |
                 ((b & 128) >> 7) << p[7]) as usize;
    }
}

fn inv_sub(sub : &mut [usize; 256]) {
    let mut c = [0usize; 256];
    for i in 0usize..256 {
        c[i] = sub[i];
    }
    for i in 0usize..256 {
        sub[c[i]] = i;
    }
}


fn dummy_crypt_file(input : &str, output : &str, key : &Vec<Vec<u8>>) {
    let mut in_file = match File::open(input) {
        Ok(f) => { f },
        Err(e) => { println!("Failed to open input file {}: {}!", input, e); return;}
    };
    let mut out_file = match File::create(output) {
        Ok(f) => { f },
        Err(e) => { println!("Failed to open output file {}: {}!", output, e); return;}
    };
    let mut buffer = Vec::<u8>::new();
    let mut sub = Vec::<[usize; 256]>::new();
    for i in 0..key[0].len() {
        sub.push([0usize; 256]);
        gen_sub(key[0][i], key[1][i], ((key[2][2*i] as u16) << 8) + key[2][2*i+1] as u16, &mut sub[i]);
    }
    if in_file.read_to_end(&mut buffer).is_err() {
        println!("Failed to read input file.");
        return;
    }
    for i in 0..buffer.len() {
        buffer[i] = sub[i%sub.len()][buffer[i] as usize] as u8;
    }
    if out_file.write_all(&buffer[..]).is_err() {
        println!("Failed to write output file.");
    }
}

fn dummy_decrypt_file(input : &str, output : &str, key : &Vec<Vec<u8>>) {
    let mut in_file = match File::open(input) {
        Ok(f) => { f },
        Err(e) => { println!("Failed to open input file {}: {}!", input, e); return;}
    };
    let mut out_file = match File::create(output) {
        Ok(f) => { f },
        Err(e) => { println!("Failed to open output file {}: {}!", output, e); return;}
    };
    let mut buffer = Vec::<u8>::new();
    let mut sub = Vec::<[usize; 256]>::new();
    for i in 0..key[0].len() {
        sub.push([0usize; 256]);
        gen_sub(key[0][i], key[1][i], ((key[2][2*i] as u16) << 8) + key[2][2*i+1] as u16, &mut sub[i]);
        inv_sub(&mut sub[i]);
    }
    if in_file.read_to_end(&mut buffer).is_err() {
        println!("Failed to read input file.");
        return;
    }
    for i in 0..buffer.len() {
        buffer[i] = sub[i%sub.len()][buffer[i] as usize] as u8;
    }
    if out_file.write_all(&buffer[..]).is_err() {
        println!("Failed to write encrypted file.");
    }
}
