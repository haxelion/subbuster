#SubBuster#

SubBuster is an improved version of 
[xortool](https://github.com/hellman/xortool) written in rust. The main 
differences are:

* The key lenght detection uses entropy instead of just counting the number of 
equal bytes.
* It tries to minimize the error on the whole byte frequency distribution 
instead of just the most frequent character.
* It should work on something else than ASCII.
* It aims to break any byte level substitution cipher, not just xor cipher. 
Currently only xor, xor-add, xor-add-mix substitutions are implemented.
* It's parallelized

For an example of the crypto it aims to break, see DummyCrypt.

## Compilation ##

You need the latest [cargo](https://crates.io) and 
[rust](http://www.rust-lang.org/) nightlies. Although it's a bit annoying, the 
easiest way is by running:

```sh
curl -sS https://static.rust-lang.org/rustup.sh | sudo bash
```

This will change when rust 1.0 is out.

To compile:

```sh
cd subbuster
cargo build --release
```

The resulting binary will be target/release/subbuster.
 
## Usage ##

subbuster [-m [1|2|3]] [-l l] [-v] input sample

* input: input file to decipher.
* sample: some plaintext sample from which byte the frequency distribution is 
computed.
* -m: optional model level number, default to 1. Model level 1 is xor, model 
level 2 is xor-add, model level 3 is xor-add-mix.
* -l: optional key lenght. If not provided, subbuster attempts to guess the key 
lenght using entropy.
* -v: verbose mode, the results from all the candidates

Warning: model level 3 is really slow (a few hours on a modern computer) 
because it attempts to bruteforce all 2 642 411 520 key possibilites per byte. 
A faster algorithm is planned.

## Why rust? ##

I know it's a pain to install a compiler and runtime just for this program.

I just wanted to try a new modern language and needed a real project to do so. 
Rust is theoritically full of advantages: type safety, memory safety, safe 
concurency patterns but still compiles to a native binary. I just wanted to 
see if it was practical to use for a real project (not like hum ... haskell).

The answer is yes. So maybe you should try rust too.

## License ##

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
