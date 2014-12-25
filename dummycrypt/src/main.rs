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

use std::os;
use std::vec::Vec;
use std::cmp::max;
use std::io::File;

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
    println!("Copyright 2014 Charles Hubain <github@haxelion.eu>");
}

enum Mode {Missing, Encrypt, Decrypt}

fn main() {
    let mut args: Vec<String> = os::args();
    if args.len() < 4 {
        print_usage();
        return;
    }
    let output = args.pop().unwrap();
    let input = args.pop().unwrap();
    let mut mode : Mode = Mode::Missing;
    let mut key : Vec<Vec<u8>> = Vec::from_elem(3, Vec::<u8>::new());
    let mut i = 1u;
    while i < args.len() {
        let arg = args[i].as_slice();
        if arg == "-e" {
            mode = Mode::Encrypt;
        }
        else if arg == "-d" {
            mode = Mode::Decrypt;
        }
        else if arg == "-x" {
            i += 1;
            if i >= args.len() {
                println!("You need to provide a xor hex string after -x");
                print_usage();
                return;
            }
            hex_to_bytes(args[i].as_slice(), &mut key[0]);
        }
        else if arg == "-a" {
            i += 1;
            if i >= args.len() {
                println!("You need to provide an add hex string after -a");
                print_usage();
                return;
            }
            hex_to_bytes(args[i].as_slice(), &mut key[1]);
        }
        else if arg == "-m" {
            i += 1;
            if i >= args.len() {
                println!("You need to provide a mix hex string after -m");
                print_usage();
                return;
            }
            hex_to_bytes(args[i].as_slice(), &mut key[2]);
        }
        else {
            print_usage();
            return;
        }
        i += 1;
    }
    let lenght = max(key[0].len(), max(key[1].len(), key[2].len()/2));
    key[0].resize(lenght, 0u8);
    key[1].resize(lenght, 0u8);
    key[2].resize(lenght*2, 0u8);
    match mode {
        Mode::Encrypt => dummy_crypt_file(input.as_slice(), output.as_slice(), &key),
        Mode::Decrypt => dummy_decrypt_file(input.as_slice(), output.as_slice(), &key),
        Mode::Missing => print_usage(),
    };
}

fn hex_to_bytes(h : &str, b :  &mut Vec<u8>) {
    let mut i = 0u;
    while i < h.len()-1 {
        let n1 = match char_to_nibble(h.char_at(i)) {
            Some(n) => { n },
            None => { println!("Hex string {} is invalid!", h); return; }
        };
        let n2 = match char_to_nibble(h.char_at(i+1)) {
            Some(n) => { n },
            None => { println!("Hex string {} is invalid!", h); return; }
        };
        b.push((n1 << 4) + n2);
        i += 2;
    }
}

fn char_to_nibble(c : char) -> Option<u8> {
    return match c {
        '0' => Some(0),
        '1' => Some(1),
        '2' => Some(2),
        '3' => Some(3),
        '4' => Some(4),
        '5' => Some(5),
        '6' => Some(6),
        '7' => Some(7),
        '8' => Some(8),
        '9' => Some(9),
        'a' => Some(10),
        'A' => Some(10),
        'b' => Some(11),
        'B' => Some(11),
        'c' => Some(12),
        'C' => Some(12),
        'd' => Some(13),
        'D' => Some(13),
        'e' => Some(14),
        'E' => Some(14),
        'f' => Some(15),
        'F' => Some(15),
        _ => None
    }
}

fn gen_sub(x : u8, a : u8, m : u16, sub : &mut [uint, ..256]) {
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

fn inv_sub(sub : &mut [uint, ..256]) {
    let mut c = [0u, ..256];
    for i in range(0u, 256) {
        c[i] = sub[i];
    }
    for i in range(0u, 256) {
        sub[c[i]] = i;
    }
}


fn dummy_crypt_file(input : &str, output : &str, key : &Vec<Vec<u8>>) {
    let mut in_file = match File::open(&Path::new(input)) {
        Ok(f) => { f },
        Err(e) => { println!("Failed to open input file {}: {}!", input, e); return;}
    };
    let mut out_file = match File::create(&Path::new(output)) {
        Ok(f) => { f },
        Err(e) => { println!("Failed to open output file {}: {}!", output, e); return;}
    };
    let mut buffer : [u8, ..1024] = [0, ..1024];
    let mut sub : Vec<[uint, ..256]> = Vec::new();
    for i in range(0u, key[0].len()) {
        sub.push([0u, ..256]);
        gen_sub(key[0][i], key[1][i], (key[2][2*i] as u16 << 8) + key[2][2*i+1] as u16, &mut sub[i]);
    }
    let mut j = 0u;
    loop {
        let n = match in_file.read(&mut buffer) {
            Ok(n) => { n },
            Err(_) => { break; }
        };
        for i in range(0, n) {
            buffer[i] = sub[j%sub.len()][buffer[i] as uint] as u8;
            j += 1;
        }
        out_file.write(buffer.slice(0, n));
    }
}

fn dummy_decrypt_file(input : &str, output : &str, key : &Vec<Vec<u8>>) {
    let mut in_file = match File::open(&Path::new(input)) {
        Ok(f) => { f },
        Err(e) => { println!("Failed to open input file {}: {}!", input, e); return;}
    };
    let mut out_file = match File::create(&Path::new(output)) {
        Ok(f) => { f },
        Err(e) => { println!("Failed to open output file {}: {}!", output, e); return;}
    };
    let mut buffer : [u8, ..1024] = [0, ..1024];
    let mut sub : Vec<[uint, ..256]> = Vec::new();
    for i in range(0u, key[0].len()) {
        sub.push([0u, ..256]);
        gen_sub(key[0][i], key[1][i], (key[2][2*i] as u16 << 8) + key[2][2*i+1] as u16, &mut sub[i]);
        inv_sub(&mut sub[i]);
    }
    let mut j = 0u;
    loop {
        let n = match in_file.read(&mut buffer) {
            Ok(n) => { n },
            Err(_) => { break; }
        };
        for i in range(0, n) {
            buffer[i] = sub[j%sub.len()][buffer[i] as uint] as u8;
            j += 1;
        }
        out_file.write(buffer.slice(0, n));
    }
}
