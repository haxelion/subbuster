# DummyCrypt #

*THIS IS UNDER NO CIRCUMSTANCE AN EFFECTIVE ENCRYPTION SOFTWARE*

*THIS PROGRAM IS DEEPLY FLAWED BY DESIGN*

*ITS ONLY USE IS TO GENERATE BAD CRYPTO TEST VECTORS*

## Usage ##

dummycrypt (-e|-d) [-x X] [-a A] [-m M] input output

* -e: specify encryption mode
* -d: specify decryption mode
* -x: optional xor hex string of bytes
* -a: optional add hex string of bytes
* -m: optional mix hex string of big endian 16 bits unsigned integer
* input: input file name
* output: output file name

The hex strings are padded with zeroes to the same number of elements.

The elements of M represent any of the 40320 possible bijective bit mix 
operations, their encoding is described below.

The cipher encryption algorithm for each byte b is  MIX(ADD(XOR(b,x),a),m)
where x, a, m are elements taken from X, A and M respectively and wrap around 
when the input is bigger than the key.

## Bit Mix ##

Because the bit mix operation has to be bijective (~invertible) there are only 
8\*7\*6\*5\*4\*3\*2\*1 = 40320 possibilites and the most effective encoding 
takes 16 bits. The first bit can be placed in 8 different places, the second 
one only has 7 possibilities left, etc. The last bit has his new position 
determined by the all the previous one. If we call those position possibilities 
p0 to p7, the bit mix operation number is:

m = p0\*7\*6\*5\*4\*3\*2 + p1\*6\*5\*4\*3\*2 + p2\*5\*4\*3\*2 + p3\*4\*3\*2 +
p4\*3\*2 + p5\*2 + p6

### Examples ###

Here are the bit mix number for the different left rotation operation:

* rol 1 : 5913  : 0x1719
* rol 2 : 11824 : 0x2e30
* rol 3 : 17730 : 0x4542
* rol 4 : 23616 : 0x5c40
* rol 5 : 29400 : 0x72d8
* rol 6 : 34560 : 0x8700
* rol 7 : 35280 : 0x89d0

## License ##

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
