/*
 * This version has been further modified by Rich Felker, primary author
 * and maintainer of musl libc, to remove table generation code and
 * replaced all runtime-generated constant tables with static-initialized
 * tables in the binary, in the interest of minimizing non-shareable
 * memory usage and stack size requirements.
 */
/*
 * This version is derived from the original implementation of FreeSec
 * (release 1.1) by David Burren.  I've made it reentrant, reduced its memory
 * usage from about 70 KB to about 7 KB (with only minimal performance impact
 * and keeping code size about the same), made the handling of invalid salts
 * mostly UFC-crypt compatible, added a quick runtime self-test (which also
 * serves to zeroize the stack from sensitive data), and added optional tests.
 * - Solar Designer <solar at openwall.com>
 */

/*
 * FreeSec: libcrypt for NetBSD
 *
 * Copyright (c) 1994 David Burren
 * Copyright (c) 2000,2002,2010,2012 Solar Designer
 * All rights reserved.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions
 * are met:
 * 1. Redistributions of source code must retain the above copyright
 *    notice, this list of conditions and the following disclaimer.
 * 2. Redistributions in binary form must reproduce the above copyright
 *    notice, this list of conditions and the following disclaimer in the
 *    documentation and/or other materials provided with the distribution.
 * 3. Neither the name of the author nor the names of other contributors
 *    may be used to endorse or promote products derived from this software
 *    without specific prior written permission.
 *
 * THIS SOFTWARE IS PROVIDED BY THE AUTHOR AND CONTRIBUTORS ``AS IS'' AND
 * ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
 * IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE
 * ARE DISCLAIMED.  IN NO EVENT SHALL THE AUTHOR OR CONTRIBUTORS BE LIABLE
 * FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
 * DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS
 * OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION)
 * HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT
 * LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY
 * OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF
 * SUCH DAMAGE.
 *
 *    $Owl: Owl/packages/glibc/crypt_freesec.c,v 1.6 2010/02/20 14:45:06 solar Exp $
 *    $Id: crypt.c,v 1.15 1994/09/13 04:58:49 davidb Exp $
 *
 * This is an original implementation of the DES and the crypt(3) interfaces
 * by David Burren.  It has been heavily re-worked by Solar Designer.
 */

use std::num::Wrapping;
use std::str;

struct ExpandedKey {
    l: [u32; 16],
    r: [u32; 16],
}

const KEY_SHIFTS: [u8; 16] = [1, 1, 2, 2, 2, 2, 2, 2, 1, 2, 2, 2, 2, 2, 2, 1];

const PSBOX: [[u32; 64]; 8] =
    [[0x00808200, 0x00000000, 0x00008000, 0x00808202, 0x00808002, 0x00008202, 0x00000002,
      0x00008000, 0x00000200, 0x00808200, 0x00808202, 0x00000200, 0x00800202, 0x00808002,
      0x00800000, 0x00000002, 0x00000202, 0x00800200, 0x00800200, 0x00008200, 0x00008200,
      0x00808000, 0x00808000, 0x00800202, 0x00008002, 0x00800002, 0x00800002, 0x00008002,
      0x00000000, 0x00000202, 0x00008202, 0x00800000, 0x00008000, 0x00808202, 0x00000002,
      0x00808000, 0x00808200, 0x00800000, 0x00800000, 0x00000200, 0x00808002, 0x00008000,
      0x00008200, 0x00800002, 0x00000200, 0x00000002, 0x00800202, 0x00008202, 0x00808202,
      0x00008002, 0x00808000, 0x00800202, 0x00800002, 0x00000202, 0x00008202, 0x00808200,
      0x00000202, 0x00800200, 0x00800200, 0x00000000, 0x00008002, 0x00008200, 0x00000000,
      0x00808002],
     [0x40084010, 0x40004000, 0x00004000, 0x00084010, 0x00080000, 0x00000010, 0x40080010,
      0x40004010, 0x40000010, 0x40084010, 0x40084000, 0x40000000, 0x40004000, 0x00080000,
      0x00000010, 0x40080010, 0x00084000, 0x00080010, 0x40004010, 0x00000000, 0x40000000,
      0x00004000, 0x00084010, 0x40080000, 0x00080010, 0x40000010, 0x00000000, 0x00084000,
      0x00004010, 0x40084000, 0x40080000, 0x00004010, 0x00000000, 0x00084010, 0x40080010,
      0x00080000, 0x40004010, 0x40080000, 0x40084000, 0x00004000, 0x40080000, 0x40004000,
      0x00000010, 0x40084010, 0x00084010, 0x00000010, 0x00004000, 0x40000000, 0x00004010,
      0x40084000, 0x00080000, 0x40000010, 0x00080010, 0x40004010, 0x40000010, 0x00080010,
      0x00084000, 0x00000000, 0x40004000, 0x00004010, 0x40000000, 0x40080010, 0x40084010,
      0x00084000],
     [0x00000104, 0x04010100, 0x00000000, 0x04010004, 0x04000100, 0x00000000, 0x00010104,
      0x04000100, 0x00010004, 0x04000004, 0x04000004, 0x00010000, 0x04010104, 0x00010004,
      0x04010000, 0x00000104, 0x04000000, 0x00000004, 0x04010100, 0x00000100, 0x00010100,
      0x04010000, 0x04010004, 0x00010104, 0x04000104, 0x00010100, 0x00010000, 0x04000104,
      0x00000004, 0x04010104, 0x00000100, 0x04000000, 0x04010100, 0x04000000, 0x00010004,
      0x00000104, 0x00010000, 0x04010100, 0x04000100, 0x00000000, 0x00000100, 0x00010004,
      0x04010104, 0x04000100, 0x04000004, 0x00000100, 0x00000000, 0x04010004, 0x04000104,
      0x00010000, 0x04000000, 0x04010104, 0x00000004, 0x00010104, 0x00010100, 0x04000004,
      0x04010000, 0x04000104, 0x00000104, 0x04010000, 0x00010104, 0x00000004, 0x04010004,
      0x00010100],
     [0x80401000, 0x80001040, 0x80001040, 0x00000040, 0x00401040, 0x80400040, 0x80400000,
      0x80001000, 0x00000000, 0x00401000, 0x00401000, 0x80401040, 0x80000040, 0x00000000,
      0x00400040, 0x80400000, 0x80000000, 0x00001000, 0x00400000, 0x80401000, 0x00000040,
      0x00400000, 0x80001000, 0x00001040, 0x80400040, 0x80000000, 0x00001040, 0x00400040,
      0x00001000, 0x00401040, 0x80401040, 0x80000040, 0x00400040, 0x80400000, 0x00401000,
      0x80401040, 0x80000040, 0x00000000, 0x00000000, 0x00401000, 0x00001040, 0x00400040,
      0x80400040, 0x80000000, 0x80401000, 0x80001040, 0x80001040, 0x00000040, 0x80401040,
      0x80000040, 0x80000000, 0x00001000, 0x80400000, 0x80001000, 0x00401040, 0x80400040,
      0x80001000, 0x00001040, 0x00400000, 0x80401000, 0x00000040, 0x00400000, 0x00001000,
      0x00401040],
     [0x00000080, 0x01040080, 0x01040000, 0x21000080, 0x00040000, 0x00000080, 0x20000000,
      0x01040000, 0x20040080, 0x00040000, 0x01000080, 0x20040080, 0x21000080, 0x21040000,
      0x00040080, 0x20000000, 0x01000000, 0x20040000, 0x20040000, 0x00000000, 0x20000080,
      0x21040080, 0x21040080, 0x01000080, 0x21040000, 0x20000080, 0x00000000, 0x21000000,
      0x01040080, 0x01000000, 0x21000000, 0x00040080, 0x00040000, 0x21000080, 0x00000080,
      0x01000000, 0x20000000, 0x01040000, 0x21000080, 0x20040080, 0x01000080, 0x20000000,
      0x21040000, 0x01040080, 0x20040080, 0x00000080, 0x01000000, 0x21040000, 0x21040080,
      0x00040080, 0x21000000, 0x21040080, 0x01040000, 0x00000000, 0x20040000, 0x21000000,
      0x00040080, 0x01000080, 0x20000080, 0x00040000, 0x00000000, 0x20040000, 0x01040080,
      0x20000080],
     [0x10000008, 0x10200000, 0x00002000, 0x10202008, 0x10200000, 0x00000008, 0x10202008,
      0x00200000, 0x10002000, 0x00202008, 0x00200000, 0x10000008, 0x00200008, 0x10002000,
      0x10000000, 0x00002008, 0x00000000, 0x00200008, 0x10002008, 0x00002000, 0x00202000,
      0x10002008, 0x00000008, 0x10200008, 0x10200008, 0x00000000, 0x00202008, 0x10202000,
      0x00002008, 0x00202000, 0x10202000, 0x10000000, 0x10002000, 0x00000008, 0x10200008,
      0x00202000, 0x10202008, 0x00200000, 0x00002008, 0x10000008, 0x00200000, 0x10002000,
      0x10000000, 0x00002008, 0x10000008, 0x10202008, 0x00202000, 0x10200000, 0x00202008,
      0x10202000, 0x00000000, 0x10200008, 0x00000008, 0x00002000, 0x10200000, 0x00202008,
      0x00002000, 0x00200008, 0x10002008, 0x00000000, 0x10202000, 0x10000000, 0x00200008,
      0x10002008],
     [0x00100000, 0x02100001, 0x02000401, 0x00000000, 0x00000400, 0x02000401, 0x00100401,
      0x02100400, 0x02100401, 0x00100000, 0x00000000, 0x02000001, 0x00000001, 0x02000000,
      0x02100001, 0x00000401, 0x02000400, 0x00100401, 0x00100001, 0x02000400, 0x02000001,
      0x02100000, 0x02100400, 0x00100001, 0x02100000, 0x00000400, 0x00000401, 0x02100401,
      0x00100400, 0x00000001, 0x02000000, 0x00100400, 0x02000000, 0x00100400, 0x00100000,
      0x02000401, 0x02000401, 0x02100001, 0x02100001, 0x00000001, 0x00100001, 0x02000000,
      0x02000400, 0x00100000, 0x02100400, 0x00000401, 0x00100401, 0x02100400, 0x00000401,
      0x02000001, 0x02100401, 0x02100000, 0x00100400, 0x00000000, 0x00000001, 0x02100401,
      0x00000000, 0x00100401, 0x02100000, 0x00000400, 0x02000001, 0x02000400, 0x00000400,
      0x00100001],
     [0x08000820, 0x00000800, 0x00020000, 0x08020820, 0x08000000, 0x08000820, 0x00000020,
      0x08000000, 0x00020020, 0x08020000, 0x08020820, 0x00020800, 0x08020800, 0x00020820,
      0x00000800, 0x00000020, 0x08020000, 0x08000020, 0x08000800, 0x00000820, 0x00020800,
      0x00020020, 0x08020020, 0x08020800, 0x00000820, 0x00000000, 0x00000000, 0x08020020,
      0x08000020, 0x08000800, 0x00020820, 0x00020000, 0x00020820, 0x00020000, 0x08020800,
      0x00000800, 0x00000020, 0x08020020, 0x00000800, 0x00020820, 0x08000800, 0x00000020,
      0x08000020, 0x08020000, 0x08020020, 0x08000000, 0x00020000, 0x08000820, 0x00000000,
      0x08020820, 0x00020020, 0x08000020, 0x08020000, 0x08000800, 0x08000820, 0x00000000,
      0x08020820, 0x00020800, 0x00020800, 0x00000820, 0x00000820, 0x00020020, 0x08000000,
      0x08020800]];

const KEY_PERM_MASKL: [[u32; 16]; 8] =
    [[0x00000000, 0x00000000, 0x00000010, 0x00000010, 0x00001000, 0x00001000, 0x00001010,
      0x00001010, 0x00100000, 0x00100000, 0x00100010, 0x00100010, 0x00101000, 0x00101000,
      0x00101010, 0x00101010],
     [0x00000000, 0x00000000, 0x00000020, 0x00000020, 0x00002000, 0x00002000, 0x00002020,
      0x00002020, 0x00200000, 0x00200000, 0x00200020, 0x00200020, 0x00202000, 0x00202000,
      0x00202020, 0x00202020],
     [0x00000000, 0x00000000, 0x00000040, 0x00000040, 0x00004000, 0x00004000, 0x00004040,
      0x00004040, 0x00400000, 0x00400000, 0x00400040, 0x00400040, 0x00404000, 0x00404000,
      0x00404040, 0x00404040],
     [0x00000000, 0x00000000, 0x00000080, 0x00000080, 0x00008000, 0x00008000, 0x00008080,
      0x00008080, 0x00800000, 0x00800000, 0x00800080, 0x00800080, 0x00808000, 0x00808000,
      0x00808080, 0x00808080],
     [0x00000000, 0x00000001, 0x00000100, 0x00000101, 0x00010000, 0x00010001, 0x00010100,
      0x00010101, 0x01000000, 0x01000001, 0x01000100, 0x01000101, 0x01010000, 0x01010001,
      0x01010100, 0x01010101],
     [0x00000000, 0x00000002, 0x00000200, 0x00000202, 0x00020000, 0x00020002, 0x00020200,
      0x00020202, 0x02000000, 0x02000002, 0x02000200, 0x02000202, 0x02020000, 0x02020002,
      0x02020200, 0x02020202],
     [0x00000000, 0x00000004, 0x00000400, 0x00000404, 0x00040000, 0x00040004, 0x00040400,
      0x00040404, 0x04000000, 0x04000004, 0x04000400, 0x04000404, 0x04040000, 0x04040004,
      0x04040400, 0x04040404],
     [0x00000000, 0x00000008, 0x00000800, 0x00000808, 0x00080000, 0x00080008, 0x00080800,
      0x00080808, 0x08000000, 0x08000008, 0x08000800, 0x08000808, 0x08080000, 0x08080008,
      0x08080800, 0x08080808]];

const KEY_PERM_MASKR: [[u32; 16]; 12] =
    [[0x00000000, 0x00000001, 0x00000000, 0x00000001, 0x00000000, 0x00000001, 0x00000000,
      0x00000001, 0x00000000, 0x00000001, 0x00000000, 0x00000001, 0x00000000, 0x00000001,
      0x00000000, 0x00000001],
     [0x00000000, 0x00000000, 0x00100000, 0x00100000, 0x00001000, 0x00001000, 0x00101000,
      0x00101000, 0x00000010, 0x00000010, 0x00100010, 0x00100010, 0x00001010, 0x00001010,
      0x00101010, 0x00101010],
     [0x00000000, 0x00000002, 0x00000000, 0x00000002, 0x00000000, 0x00000002, 0x00000000,
      0x00000002, 0x00000000, 0x00000002, 0x00000000, 0x00000002, 0x00000000, 0x00000002,
      0x00000000, 0x00000002],
     [0x00000000, 0x00000000, 0x00200000, 0x00200000, 0x00002000, 0x00002000, 0x00202000,
      0x00202000, 0x00000020, 0x00000020, 0x00200020, 0x00200020, 0x00002020, 0x00002020,
      0x00202020, 0x00202020],
     [0x00000000, 0x00000004, 0x00000000, 0x00000004, 0x00000000, 0x00000004, 0x00000000,
      0x00000004, 0x00000000, 0x00000004, 0x00000000, 0x00000004, 0x00000000, 0x00000004,
      0x00000000, 0x00000004],
     [0x00000000, 0x00000000, 0x00400000, 0x00400000, 0x00004000, 0x00004000, 0x00404000,
      0x00404000, 0x00000040, 0x00000040, 0x00400040, 0x00400040, 0x00004040, 0x00004040,
      0x00404040, 0x00404040],
     [0x00000000, 0x00000008, 0x00000000, 0x00000008, 0x00000000, 0x00000008, 0x00000000,
      0x00000008, 0x00000000, 0x00000008, 0x00000000, 0x00000008, 0x00000000, 0x00000008,
      0x00000000, 0x00000008],
     [0x00000000, 0x00000000, 0x00800000, 0x00800000, 0x00008000, 0x00008000, 0x00808000,
      0x00808000, 0x00000080, 0x00000080, 0x00800080, 0x00800080, 0x00008080, 0x00008080,
      0x00808080, 0x00808080],
     [0x00000000, 0x00000000, 0x01000000, 0x01000000, 0x00010000, 0x00010000, 0x01010000,
      0x01010000, 0x00000100, 0x00000100, 0x01000100, 0x01000100, 0x00010100, 0x00010100,
      0x01010100, 0x01010100],
     [0x00000000, 0x00000000, 0x02000000, 0x02000000, 0x00020000, 0x00020000, 0x02020000,
      0x02020000, 0x00000200, 0x00000200, 0x02000200, 0x02000200, 0x00020200, 0x00020200,
      0x02020200, 0x02020200],
     [0x00000000, 0x00000000, 0x04000000, 0x04000000, 0x00040000, 0x00040000, 0x04040000,
      0x04040000, 0x00000400, 0x00000400, 0x04000400, 0x04000400, 0x00040400, 0x00040400,
      0x04040400, 0x04040400],
     [0x00000000, 0x00000000, 0x08000000, 0x08000000, 0x00080000, 0x00080000, 0x08080000,
      0x08080000, 0x00000800, 0x00000800, 0x08000800, 0x08000800, 0x00080800, 0x00080800,
      0x08080800, 0x08080800]];

const COMP_MASKL0: [[u32; 8]; 4] = [[0x00000000, 0x00020000, 0x00000001, 0x00020001, 0x00080000,
                                     0x000a0000, 0x00080001, 0x000a0001],
                                    [0x00000000, 0x00001000, 0x00000000, 0x00001000, 0x00000040,
                                     0x00001040, 0x00000040, 0x00001040],
                                    [0x00000000, 0x00400000, 0x00000020, 0x00400020, 0x00008000,
                                     0x00408000, 0x00008020, 0x00408020],
                                    [0x00000000, 0x00100000, 0x00000800, 0x00100800, 0x00000000,
                                     0x00100000, 0x00000800, 0x00100800]];

const COMP_MASKR0: [[u32; 8]; 4] = [[0x00000000, 0x00200000, 0x00020000, 0x00220000, 0x00000002,
                                     0x00200002, 0x00020002, 0x00220002],
                                    [0x00000000, 0x00000000, 0x00100000, 0x00100000, 0x00000004,
                                     0x00000004, 0x00100004, 0x00100004],
                                    [0x00000000, 0x00004000, 0x00000800, 0x00004800, 0x00000000,
                                     0x00004000, 0x00000800, 0x00004800],
                                    [0x00000000, 0x00400000, 0x00008000, 0x00408000, 0x00000008,
                                     0x00400008, 0x00008008, 0x00408008]];

const COMP_MASKL1: [[u32; 16]; 4] =
    [[0x00000000, 0x00000010, 0x00004000, 0x00004010, 0x00040000, 0x00040010, 0x00044000,
      0x00044010, 0x00000100, 0x00000110, 0x00004100, 0x00004110, 0x00040100, 0x00040110,
      0x00044100, 0x00044110],
     [0x00000000, 0x00800000, 0x00000002, 0x00800002, 0x00000200, 0x00800200, 0x00000202,
      0x00800202, 0x00200000, 0x00a00000, 0x00200002, 0x00a00002, 0x00200200, 0x00a00200,
      0x00200202, 0x00a00202],
     [0x00000000, 0x00002000, 0x00000004, 0x00002004, 0x00000400, 0x00002400, 0x00000404,
      0x00002404, 0x00000000, 0x00002000, 0x00000004, 0x00002004, 0x00000400, 0x00002400,
      0x00000404, 0x00002404],
     [0x00000000, 0x00010000, 0x00000008, 0x00010008, 0x00000080, 0x00010080, 0x00000088,
      0x00010088, 0x00000000, 0x00010000, 0x00000008, 0x00010008, 0x00000080, 0x00010080,
      0x00000088, 0x00010088]];


const COMP_MASKR1: [[u32; 16]; 4] =
    [[0x00000000, 0x00000000, 0x00000080, 0x00000080, 0x00002000, 0x00002000, 0x00002080,
      0x00002080, 0x00000001, 0x00000001, 0x00000081, 0x00000081, 0x00002001, 0x00002001,
      0x00002081, 0x00002081],
     [0x00000000, 0x00000010, 0x00800000, 0x00800010, 0x00010000, 0x00010010, 0x00810000,
      0x00810010, 0x00000200, 0x00000210, 0x00800200, 0x00800210, 0x00010200, 0x00010210,
      0x00810200, 0x00810210],
     [0x00000000, 0x00000400, 0x00001000, 0x00001400, 0x00080000, 0x00080400, 0x00081000,
      0x00081400, 0x00000020, 0x00000420, 0x00001020, 0x00001420, 0x00080020, 0x00080420,
      0x00081020, 0x00081420],
     [0x00000000, 0x00000100, 0x00040000, 0x00040100, 0x00000000, 0x00000100, 0x00040000,
      0x00040100, 0x00000040, 0x00000140, 0x00040040, 0x00040140, 0x00000040, 0x00000140,
      0x00040040, 0x00040140]];

const FP_MASKL: [[u32; 16]; 8] =
    [[0x00000000, 0x40000000, 0x00400000, 0x40400000, 0x00004000, 0x40004000, 0x00404000,
      0x40404000, 0x00000040, 0x40000040, 0x00400040, 0x40400040, 0x00004040, 0x40004040,
      0x00404040, 0x40404040],
     [0x00000000, 0x10000000, 0x00100000, 0x10100000, 0x00001000, 0x10001000, 0x00101000,
      0x10101000, 0x00000010, 0x10000010, 0x00100010, 0x10100010, 0x00001010, 0x10001010,
      0x00101010, 0x10101010],
     [0x00000000, 0x04000000, 0x00040000, 0x04040000, 0x00000400, 0x04000400, 0x00040400,
      0x04040400, 0x00000004, 0x04000004, 0x00040004, 0x04040004, 0x00000404, 0x04000404,
      0x00040404, 0x04040404],
     [0x00000000, 0x01000000, 0x00010000, 0x01010000, 0x00000100, 0x01000100, 0x00010100,
      0x01010100, 0x00000001, 0x01000001, 0x00010001, 0x01010001, 0x00000101, 0x01000101,
      0x00010101, 0x01010101],
     [0x00000000, 0x80000000, 0x00800000, 0x80800000, 0x00008000, 0x80008000, 0x00808000,
      0x80808000, 0x00000080, 0x80000080, 0x00800080, 0x80800080, 0x00008080, 0x80008080,
      0x00808080, 0x80808080],
     [0x00000000, 0x20000000, 0x00200000, 0x20200000, 0x00002000, 0x20002000, 0x00202000,
      0x20202000, 0x00000020, 0x20000020, 0x00200020, 0x20200020, 0x00002020, 0x20002020,
      0x00202020, 0x20202020],
     [0x00000000, 0x08000000, 0x00080000, 0x08080000, 0x00000800, 0x08000800, 0x00080800,
      0x08080800, 0x00000008, 0x08000008, 0x00080008, 0x08080008, 0x00000808, 0x08000808,
      0x00080808, 0x08080808],
     [0x00000000, 0x02000000, 0x00020000, 0x02020000, 0x00000200, 0x02000200, 0x00020200,
      0x02020200, 0x00000002, 0x02000002, 0x00020002, 0x02020002, 0x00000202, 0x02000202,
      0x00020202, 0x02020202]];

const FP_MASKR: [[u32; 16]; 8] =
    [[0x00000000, 0x40000000, 0x00400000, 0x40400000, 0x00004000, 0x40004000, 0x00404000,
      0x40404000, 0x00000040, 0x40000040, 0x00400040, 0x40400040, 0x00004040, 0x40004040,
      0x00404040, 0x40404040],
     [0x00000000, 0x10000000, 0x00100000, 0x10100000, 0x00001000, 0x10001000, 0x00101000,
      0x10101000, 0x00000010, 0x10000010, 0x00100010, 0x10100010, 0x00001010, 0x10001010,
      0x00101010, 0x10101010],
     [0x00000000, 0x04000000, 0x00040000, 0x04040000, 0x00000400, 0x04000400, 0x00040400,
      0x04040400, 0x00000004, 0x04000004, 0x00040004, 0x04040004, 0x00000404, 0x04000404,
      0x00040404, 0x04040404],
     [0x00000000, 0x01000000, 0x00010000, 0x01010000, 0x00000100, 0x01000100, 0x00010100,
      0x01010100, 0x00000001, 0x01000001, 0x00010001, 0x01010001, 0x00000101, 0x01000101,
      0x00010101, 0x01010101],
     [0x00000000, 0x80000000, 0x00800000, 0x80800000, 0x00008000, 0x80008000, 0x00808000,
      0x80808000, 0x00000080, 0x80000080, 0x00800080, 0x80800080, 0x00008080, 0x80008080,
      0x00808080, 0x80808080],
     [0x00000000, 0x20000000, 0x00200000, 0x20200000, 0x00002000, 0x20002000, 0x00202000,
      0x20202000, 0x00000020, 0x20000020, 0x00200020, 0x20200020, 0x00002020, 0x20002020,
      0x00202020, 0x20202020],
     [0x00000000, 0x08000000, 0x00080000, 0x08080000, 0x00000800, 0x08000800, 0x00080800,
      0x08080800, 0x00000008, 0x08000008, 0x00080008, 0x08080008, 0x00000808, 0x08000808,
      0x00080808, 0x08080808],
     [0x00000000, 0x02000000, 0x00020000, 0x02020000, 0x00000200, 0x02000200, 0x00020200,
      0x02020200, 0x00000002, 0x02000002, 0x00020002, 0x02020002, 0x00000202, 0x02000202,
      0x00020202, 0x02020202]];

const ASCII64: [u8; 64] = [0x2e, 0x2f, 0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39,
                           0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 0x49, 0x4a, 0x4b, 0x4c,
                           0x4d, 0x4e, 0x4f, 0x50, 0x51, 0x52, 0x53, 0x54, 0x55, 0x56, 0x57, 0x58,
                           0x59, 0x5a, 0x61, 0x62, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69, 0x6a,
                           0x6b, 0x6c, 0x6d, 0x6e, 0x6f, 0x70, 0x71, 0x72, 0x73, 0x74, 0x75, 0x76,
                           0x77, 0x78, 0x79, 0x7a];

fn ascii_to_bin(ch: i32) -> u32 {
    let sch = if ch < 0x80 { ch } else { -(0x100 - ch) };

    let retval = if sch >= 'A' as i32 {
        if sch >= 'a' as i32 {
            sch - ('a' as i32 - 38)
        } else {
            sch - ('A' as i32 - 12)
        }
    } else {
        sch - '.' as i32
    };

    retval as u32 & 0x3f
}

pub fn trip(passwd: &str) -> String {
    let mut keybuf = [0u8; 8];

    for (i, val) in passwd.bytes().take(keybuf.len()).enumerate() {
        keybuf[i] = val << 1;
    }

    let mut ekey = ExpandedKey {
        l: [0; 16],
        r: [0; 16],
    };

    let rawkey0 = keybuf[3] as u32 | (keybuf[2] as u32) << 8 | (keybuf[1] as u32) << 16 |
                  (keybuf[0] as u32) << 24;

    let rawkey1 = keybuf[7] as u32 | (keybuf[6] as u32) << 8 | (keybuf[5] as u32) << 16 |
                  (keybuf[4] as u32) << 24;

    let mut k0 = 0u32;
    let mut k1 = 0u32;
    let mut ibit = 28usize;

    for i in 0usize..4 {
        let j = i << 1;

        k0 |= KEY_PERM_MASKL[i][rawkey0 as usize >> ibit & 0xf] |
              KEY_PERM_MASKL[i + 4][rawkey1 as usize >> ibit & 0xf];

        k1 |= KEY_PERM_MASKR[j][rawkey0 as usize >> ibit & 0xf];
        ibit -= 4;

        k1 |= KEY_PERM_MASKR[j + 1][rawkey0 as usize >> ibit & 0xf] |
              KEY_PERM_MASKR[i + 8][rawkey1 as usize >> ibit & 0xf];

        ibit = (Wrapping(ibit) - Wrapping(4)).0;
    }

    let mut shifts = 0usize;

    for round in 0usize..16 {
        shifts += KEY_SHIFTS[round] as usize;

        let t0 = k0 << shifts | k0 >> 28 - shifts;
        let t1 = k1 << shifts | k1 >> 28 - shifts;

        let mut kl = 0u32;
        let mut kr = 0u32;
        let mut ibit = 25usize;

        for i in 0usize..4 {
            kl |= COMP_MASKL0[i][t0 as usize >> ibit & 7];
            kr |= COMP_MASKR0[i][t1 as usize >> ibit & 7];
            ibit -= 4;
            kl |= COMP_MASKL1[i][t0 as usize >> ibit & 0xf];
            kr |= COMP_MASKR1[i][t1 as usize >> ibit & 0xf];
            ibit = (Wrapping(ibit) - Wrapping(3)).0;
        }

        ekey.l[round] = kl;
        ekey.r[round] = kr;
    }

    let mut salt_chars = passwd
        .chars()
        .chain("H.".chars())
        .skip(1)
        .map(|c| match c {
                 '/' | '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' | 'A' | 'B' |
                 'C' | 'D' | 'E' | 'F' | 'G' | 'H' | 'I' | 'J' | 'K' | 'L' | 'M' | 'N' | 'O' |
                 'P' | 'Q' | 'R' | 'S' | 'T' | 'U' | 'V' | 'W' | 'X' | 'Y' | 'Z' | 'a' | 'b' |
                 'c' | 'd' | 'e' | 'f' | 'g' | 'h' | 'i' | 'j' | 'k' | 'l' | 'm' | 'n' | 'o' |
                 'p' | 'q' | 'r' | 's' | 't' | 'u' | 'v' | 'w' | 'x' | 'y' | 'z' => c,
                 ':' => 'A',
                 ';' => 'B',
                 '<' => 'C',
                 '=' => 'D',
                 '>' => 'E',
                 '?' => 'F',
                 '@' => 'G',
                 '[' => 'a',
                 '\\' => 'b',
                 ']' => 'c',
                 '^' => 'd',
                 '_' => 'e',
                 '`' => 'f',
                 _ => '.',
             });

    let setting0 = salt_chars.next().unwrap();
    let setting1 = salt_chars.next().unwrap();
    let salt = ascii_to_bin(setting1 as i32) << 6 | ascii_to_bin(setting0 as i32);
    let mut saltbits = 0u32;
    let mut saltbit = 1u32;
    let mut obit = 0x800000;

    for _ in 0..24 {
        if salt & saltbit != 0 {
            saltbits |= obit;
        }

        saltbit <<= 1;
        obit >>= 1;
    }

    let mut l = 0u32;
    let mut r = 0u32;

    for _ in 0..25 {
        let mut f = 0u32;

        for (kl, kr) in ekey.l.iter().zip(ekey.r.iter()) {
            let mut r48l = (r & 0x00000001) << 23 | (r & 0xf8000000) >> 9 | (r & 0x1f800000) >> 11 |
                           (r & 0x01f80000) >> 13 |
                           (r & 0x001f8000) >> 15;

            let mut r48r = (r & 0x0001f800) << 7 | (r & 0x00001f80) << 5 | (r & 0x000001f8) << 3 |
                           (r & 0x0000001f) << 1 |
                           (r & 0x80000000) >> 31;

            f = (r48l ^ r48r) & saltbits;
            r48l ^= f ^ kl;
            r48r ^= f ^ kr;

            f = PSBOX[0][r48l as usize >> 18] | PSBOX[1][r48l as usize >> 12 & 0x3f] |
                PSBOX[2][r48l as usize >> 6 & 0x3f] |
                PSBOX[3][r48l as usize & 0x3f] | PSBOX[4][r48r as usize >> 18] |
                PSBOX[5][r48r as usize >> 12 & 0x3f] |
                PSBOX[6][r48r as usize >> 6 & 0x3f] |
                PSBOX[7][r48r as usize & 0x3f];

            f ^= l;
            l = r;
            r = f;
        }

        r = l;
        l = f;
    }

    let mut ibit = 28usize;
    let mut r0 = 0u32;
    let mut r1 = 0u32;

    for i in 0usize..4 {
        r1 |= FP_MASKR[i][l as usize >> ibit & 0xf] | FP_MASKR[i + 4][r as usize >> ibit & 0xf];
        ibit -= 4;
        r0 |= FP_MASKL[i][l as usize >> ibit & 0xf] | FP_MASKL[i + 4][r as usize >> ibit & 0xf];
        ibit = (Wrapping(ibit) - Wrapping(4)).0;
    }

    let mut output = [0u8; 10];
    let l = (r0 as usize) >> 8;
    output[0] = ASCII64[l >> 12 & 0x3f];
    output[1] = ASCII64[l >> 6 & 0x3f];
    output[2] = ASCII64[l & 0x3f];
    let l = ((r0 as usize) << 16) | ((r1 as usize) >> 16 & 0xffff);
    output[3] = ASCII64[l >> 18 & 0x3f];
    output[4] = ASCII64[l >> 12 & 0x3f];
    output[5] = ASCII64[l >> 6 & 0x3f];
    output[6] = ASCII64[l & 0x3f];
    let l = (r1 as usize) << 2;
    output[7] = ASCII64[l >> 12 & 0x3f];
    output[8] = ASCII64[l >> 6 & 0x3f];
    output[9] = ASCII64[l & 0x3f];
    str::from_utf8(&output).unwrap().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    #[test]
    fn matches_unix() {
        assert_eq!(trip("foofoofo"), "vctoKCJ4Fk");
    }

    #[test]
    fn eight_sig_chars() {
        assert_eq!(trip("foofoofo"), trip("foofoofoo"));
    }

    #[bench]
    fn bench_trip(b: &mut Bencher) {
        b.iter(|| trip("foofoofo"));
    }
}
