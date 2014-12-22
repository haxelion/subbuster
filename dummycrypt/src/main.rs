use std::os;
use std::io::File;

fn print_usage() {
    println!("dummycrypt (-e|-d) [-x X] [-a A] input output");
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
    let mut x : Vec<u8> = Vec::new();
    let mut a : Vec<u8> = Vec::new();
    let mut i = 1u;
    while i < args.len() {
        let arg = args[i].as_slice();
        if arg == "-e" {
            mode = Mode::Encrypt;
        } else if arg == "-d" {
            mode = Mode::Decrypt;
        } else if arg == "-x" {
            i += 1;
            hex_to_bytes(args[i].as_slice(), &mut x);
        } else if arg == "-a" {
            i += 1;
            hex_to_bytes(args[i].as_slice(), &mut a);
        } else {
            print_usage();
            return;
        }
        i += 1;
    }
    if x.is_empty() {
        x.push(0u8);
    }
    if a.is_empty() {
        a.push(0u8);
    }

    match mode {
        Mode::Encrypt => dummy_crypt_file(input.as_slice(), output.as_slice(), &x, &a),
        Mode::Decrypt => dummy_decrypt_file(input.as_slice(), output.as_slice(), &x, &a),
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

fn dummy_crypt_file(input : &str, output : &str, x : &Vec<u8>, a : &Vec<u8>) {
    let mut in_file = match File::open(&Path::new(input)) {
        Ok(f) => { f },
        Err(e) => { println!("Failed to open input file {}: {}!", input, e); return;}
    };
    let mut out_file = match File::create(&Path::new(output)) {
        Ok(f) => { f },
        Err(e) => { println!("Failed to open output file {}: {}!", output, e); return;}
    };
    let mut buffer : [u8, ..1024] = [0, ..1024];
    let mut j = 0u;
    loop {
        let n = match in_file.read(&mut buffer) {
            Ok(n) => { n },
            Err(_) => { break; }
        };
        for i in range(0, n) {
            buffer[i] = (buffer[i] ^ x[j % x.len()]) + a[j % a.len()];
            j += 1;
        }
        out_file.write(buffer.slice(0, n));
    }
}

fn dummy_decrypt_file(input : &str, output : &str, x : &Vec<u8>, a : &Vec<u8>) {
    let mut in_file = match File::open(&Path::new(input)) {
        Ok(f) => { f },
        Err(e) => { println!("Failed to open input file {}: {}!", input, e); return;}
    };
    let mut out_file = match File::create(&Path::new(output)) {
        Ok(f) => { f },
        Err(e) => { println!("Failed to open output file {}: {}!", output, e); return;}
    };
    let mut buffer : [u8, ..1024] = [0, ..1024];
    let mut j = 0u;
    loop {
        let n = match in_file.read(&mut buffer) {
            Ok(n) => { n },
            Err(_) => { break; }
        };
        for i in range(0, n) {
            buffer[i] = (buffer[i] - a[j % a.len()]) ^ x[j % x.len()];
            j += 1;
        }
        out_file.write(buffer.slice(0, n));
    }
}
