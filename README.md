#SubBuster#

SubBuster is an improved version of 
[xortool](https://github.com/hellman/xortool) written in rust. The main 
differences are:

* The key length detection uses entropy instead of just counting the number of 
equal bytes.
* It tries to minimize the error on the whole byte frequency distribution 
instead of just the most frequent character.
* It should work on something else than ASCII.
* It aims to break any byte level substitution cipher, not just xor cipher. 
Currently only xor, xor-add, xor-add-mix substitutions are implemented.
* It's parallelized

For an example of the crypto it aims to break, see DummyCrypt.

## Compilation ##

You need the latest [rust](http://www.rust-lang.org/) nightlies.

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
* -l: optional key length. If not provided, subbuster attempts to guess the key 
length using entropy.
* -k: optional maximum key length, default to 10.
* -v: verbose mode, display the results from all the candidates.

Warning: model level 3 is really slow because of the large key space 
(2 642 411 520 key possibilites per byte). It is optimized to find solutions
with high score and will abort if the solutions are too bad. 

## Example ##

```sh
wget -O rust.html "http://en.wikipedia.org/wiki/Rust_%28programming_language%29"
wget -O crypto.html "http://en.wikipedia.org/wiki/Cryptography"
./dummycrypt/target/release/dummycrypt -e -x 13374242 -a deadbeef -m 0102030405060708 crypto.html crypto.ciphered
./target/release/subbuster -v -m 3 crypto.ciphered rust.html 
```

Output:

```raw
Length candidates: 
------------------

S        | l
0.155691 : 4
0.145305 : 8
0.115363 : 2
0.103409 : 6
0.098324 : 10
0.085061 : 1
0.076248 : 3
0.072502 : 5
0.070133 : 7
0.068449 : 9


Key candidates:
---------------

S        | l   | K
0.972984 :   4 : x = 13374242 a = deadbeef m = 0102030405060708
0.972671 :   8 : x = 1337424213374242 a = deadbeefdeadbeef m = 01020304050607080102030405060708
0.849390 :   2 : x = 2a06 a = 36db m = 03d66ae8
0.847706 :   6 : x = 2a062a492a06 a = 36db36df36db m = 03d66ae803d60cc403d66ae8
0.847886 :  10 : x = 2a1770062a062a054d06 a = 36d4c0db36db36da98db m = 03d646e804886ae803d66ae803d6667a286a6ae8

Best key: 0.972984 :   4 : x = 13374242 a = deadbeef m = 0102030405060708
```

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
