/*
 * Copyright (c) 2014, NVIDIA CORPORATION.  All rights reserved.
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; either version 2 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful, but WITHOUT
 * ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or
 * FITNESS FOR A PARTICULAR PURPOSE.  See the GNU General Public License for
 * more details.
 *
 * You should have received a copy of the GNU General Public License along
 * with this program; if not, write to the Free Software Foundation, Inc.,
 * 51 Franklin Street, Fifth Floor, Boston, MA  02110-1301, USA.
 */

// modified from: https://github.com/USBhost/FIRE-ICE/blob/7a2a7e376a20880f7bb840ca971b9619ecbdd1d3/arch/arm64/lib/memcpy_base.h

/* Parameters */
dstin   .req    x0
src     .req    x1
count   .req    x2

/* Return value */
ret_val .req    x0  /* Aliased with dstin */

/* Local version of dstin */
dst     .req    x3

/* Temporary reg */
tmp1    .req    x6
tmp2    .req    x7
tmp3    .req    x8

/* Registers for copying large blocks */
A_l     .req    x9
A_h     .req    x10
B_l     .req    x11
B_h     .req    x12
C_l     .req    x13
C_h     .req    x14
D_l     .req    x15
D_h     .req    x16

A_lw    .req    w9
A_hw    .req    w10
B_lw    .req    w11
B_hw    .req    w12
C_lw    .req    w13
C_hw    .req    w14
D_lw    .req    w15
D_hw    .req    w16

/*
   block                | src alignment | dst alignment | count
   ---------------------|---------------|---------------|------------------
   tail_small           |       ?       |       ?       | count < 64
   tail_by_8            |       ?       |       ?       | 8 < count < 64
   tail_small_aligned   |      >= 8     |      >= 8     | count < 64
   tail_by_8_aligned    |      >= 8     |      >= 8     | 8 < count < 64
   tail8                |       ?       |       ?       | count <= 8
   not_short            |       ?       |       ?       | 64 <= count
   load_align_8         |      != 8     |       ?       | 64 <= count
   src_aligned_8        |      >= 8     |       ?       | 64 <= count
   both_aligned         |      >= 8     |      >= 8     | 64 <= count
   both_aligned_big     |      >= 8     |      >= 8     | 64 <= count < 128
   both_aligned_huge    |      >= 8     |      >= 8     | 128 <= count
   dst_not_aligned_8    |      >= 8     |       ?       | 64 <= count
   dst_aligned_1        |      >= 8     |       1       | 64 <= count
   dst_aligned_2        |      >= 8     |       2       | 64 <= count
   dst_aligned_4        |      >= 8     |       4       | 64 <= count
*/



.globl memmove;
.globl memcpy;
.align 6
    memmove:
    memcpy:
    mov     dst, dstin

    /* Branch to backward version if src < dst */
    cmp     dst, src
    b.ge    memcpy_backward

    /* If count >= 64, use the "not short" version */
    cmp     count, #0x40
    b.ge    .memcpy_forward_not_short

/* Count < 64, src and dst alignments are unknown */
.memcpy_forward_tail_small:
    /* If the length is greater than 8, fall through to code that copies 8
       bytes at a time. Otherwise branch to code that copies the last 8
       bytes */
    cmp     count, #0x08
    b.le    .memcpy_forward_tail8

/* Copy 8 byte chunks one byte at a time until there are less than 8 bytes to
   copy. */
.memcpy_forward_tail_by_8:
    sub     count, count, #0x08
    /* Load 8 bytes before storing any bytes. The CPU's pipeline should not
       stall until the first write. */
    ldrb    A_lw, [src, #0x00]
    ldrb    A_hw, [src, #0x01]
    ldrb    B_lw, [src, #0x02]
    ldrb    B_hw, [src, #0x03]
    ldrb    C_lw, [src, #0x04]
    ldrb    C_hw, [src, #0x05]
    ldrb    D_lw, [src, #0x06]
    ldrb    D_hw, [src, #0x07]
    /* Store the values */
    strb    A_lw, [dst, #0x00]
    strb    A_hw, [dst, #0x01]
    strb    B_lw, [dst, #0x02]
    strb    B_hw, [dst, #0x03]
    strb    C_lw, [dst, #0x04]
    strb    C_hw, [dst, #0x05]
    strb    D_lw, [dst, #0x06]
    strb    D_hw, [dst, #0x07]
    add     src, src, #0x08
    add     dst, dst, #0x08
    cmp     count, #0x08
    b.ge    .memcpy_forward_tail_by_8
    cbnz    count, .memcpy_forward_tail8
    ret

/* Count < 64, src and dst alignment >= 8 */
.memcpy_forward_tail_small_aligned:
    /* If the length is greater than 8, fall through to code that copies 8
       bytes at a time. Otherwise branch to code that copies the last 8
       bytes */
    cmp     count, #0x08
    b.le    .memcpy_forward_tail8

/* Copy 8 byte chunks one byte at a time until there are less than 8 bytes to
   copy. */
.memcpy_forward_tail_by_8_aligned:
    sub     count, count, #0x08
    ldr     A_l, [src, #0x00]
    str     A_l, [dst, #0x00]
    add     src, src, #0x08
    add     dst, dst, #0x08
    cmp     count, #0x08
    b.ge    .memcpy_forward_tail_by_8_aligned
    cbnz    count, .memcpy_forward_tail8
    ret

/* Copy the last <= 8 bytes one byte at a time */
.memcpy_forward_tail8:
    cbz     count, .memcpy_forward_tail8_end
    /* Do a calculated branch into an unrolled loop in _tail8*/
    and     tmp1, count, #0xf
    adr     tmp2, .memcpy_forward__tail8
    mov     tmp3, #0x08
    sub     tmp3, tmp3, tmp1
    add     tmp1, tmp2, tmp3, LSL #0x03
    br      tmp1

/* Unrolled loop. Each unrolled iteration is 8 bytes long so that tail8 can
   branch into the middle */
.memcpy_forward__tail8:
.macro  memcpy_forward_cpy reg
    ldrb    \reg, [src], #0x01
    strb    \reg, [dst], #0x01
.endm

    memcpy_forward_cpy     A_lw
    memcpy_forward_cpy     A_hw
    memcpy_forward_cpy     B_lw
    memcpy_forward_cpy     B_hw
    memcpy_forward_cpy     C_lw
    memcpy_forward_cpy     C_hw
    memcpy_forward_cpy     D_lw
    memcpy_forward_cpy     D_hw

.memcpy_forward_tail8_end:
    ret

/* Handle count >= 64. If src is not 8-byte aligned, fall through. */
.memcpy_forward_not_short:
    neg     tmp1, src
    ands    tmp1, tmp1, #0x07
    b.eq    .memcpy_forward_src_aligned_8
    sub     count, count, tmp1

/* Align src to be 8 byte aligned by copying one byte at a time */
.memcpy_forward_load_align_8:
    memcpy_forward_cpy     A_lw
    subs    tmp1, tmp1, #0x01
    b.gt    .memcpy_forward_load_align_8

    cmp     count, #0x40
    b.lt    .memcpy_forward_tail_small

/* src is 8 byte aligned, count >= 64 */
.memcpy_forward_src_aligned_8:
    ands    tmp1, dst, #0x07
    b.ne    .memcpy_forward_dst_not_aligned_8

/* At this point, src and dest are 8 byte aligned and count >= 64 */
.memcpy_forward_both_aligned:
    /* If count >= 128, branch to the "huge" case */
    cmp     count, #0x80
    b.ge    .memcpy_forward_both_aligned_huge

/* Copy 64 aligned bytes */
.memcpy_forward_both_aligned_big:
    sub     count, count, #0x40
    ldp     A_l, A_h, [src]
    ldp     B_l, B_h, [src, #0x10]
    ldp     C_l, C_h, [src, #0x20]
    ldp     D_l, D_h, [src, #0x30]
    add     src, src, #0x40
    cmp     count, #0x40

    stp     A_l, A_h, [dst]
    stp     B_l, B_h, [dst, #0x10]
    stp     C_l, C_h, [dst, #0x20]
    stp     D_l, D_h, [dst, #0x30]
    add     dst, dst, #0x40
    b.ne    .memcpy_forward_tail_small_aligned
    ret

/* count */
.memcpy_forward_both_aligned_huge:
    /* Prime the copy pipeline by loading values into 8 registers. */
    ldp     A_l, A_h, [src, #0]
    sub     dst, dst, #0x10
    ldp     B_l, B_h, [src, #0x10]
    ldp     C_l, C_h, [src, #0x20]
    ldp     D_l, D_h, [src, #0x30]!
    subs    count, count, #0x40
1:
    /* Immediatly after storing the register, load a new value into it. This
       maximizes the number instructions between loading the value and
       storing it. The goal of that is to avoid load stalls */
    stp     A_l, A_h, [dst, #0x10]
    ldp     A_l, A_h, [src, #0x10]
    stp     B_l, B_h, [dst, #0x20]
    ldp     B_l, B_h, [src, #0x20]
    subs    count, count, #0x40
    stp     C_l, C_h, [dst, #0x30]
    ldp     C_l, C_h, [src, #0x30]
    stp     D_l, D_h, [dst, #0x40]!
    ldp     D_l, D_h, [src, #0x40]!

    cmp     count, #0x40
    b.ge    1b

    /* Drain the copy pipeline by storing the values in the 8 registers */
    stp     A_l, A_h, [dst, #0x10]
    stp     B_l, B_h, [dst, #0x20]
    stp     C_l, C_h, [dst, #0x30]
    stp     D_l, D_h, [dst, #0x40]
    add     src, src, #0x10
    add     dst, dst, #0x40 + 0x10
    cmp     count, #0x00
    b.ne    .memcpy_forward_tail_small_aligned
    ret

/* These blocks of code handle the case where src is >= 8 byte aligned but dst
   is not. Prevent non-naturally aligned stores by manually shifting the data
   before storing it. */
.memcpy_forward_dst_not_aligned_8:
    cmp     tmp1, #0x04
    b.eq    .memcpy_forward_dst_aligned_4
    tst     tmp1, #0x01
    b.eq    .memcpy_forward_dst_aligned_2

/* Load 64 bytes into 8 byte registers */
.macro  memcpy_forward_load_array, src
    ldp     A_l, A_h, [\src, #0x00]
    ldp     B_l, B_h, [\src, #0x10]
    ldp     C_l, C_h, [\src, #0x20]
    ldp     D_l, D_h, [\src, #0x30]
.endm

/* Store a value from each register in the array */
.macro  memcpy_forward_store_array, str, offset
    \str    A_lw, [dst, #0x00 + \offset]
    \str    A_hw, [dst, #0x08 + \offset]
    \str    B_lw, [dst, #0x10 + \offset]
    \str    B_hw, [dst, #0x18 + \offset]
    \str    C_lw, [dst, #0x20 + \offset]
    \str    C_hw, [dst, #0x28 + \offset]
    \str    D_lw, [dst, #0x30 + \offset]
    \str    D_hw, [dst, #0x38 + \offset]
.endm

/* Store the lower bits in the registers and then shift the data right to get
   new data in the lower bits. */
.macro  memcpy_forward_store_shift_array, str, offset, bits
    \str    A_lw, [dst, #0x00 + \offset]
    lsr     A_l, A_l, \bits
    \str    A_hw, [dst, #0x08 + \offset]
    lsr     A_h, A_h, \bits
    \str    B_lw, [dst, #0x10 + \offset]
    lsr     B_l, B_l, \bits
    \str    B_hw, [dst, #0x18 + \offset]
    lsr     B_h, B_h, \bits
    \str    C_lw, [dst, #0x20 + \offset]
    lsr     C_l, C_l, \bits
    \str    C_hw, [dst, #0x28 + \offset]
    lsr     C_h, C_h, \bits
    \str    D_lw, [dst, #0x30 + \offset]
    lsr     D_l, D_l, \bits
    \str    D_hw, [dst, #0x38 + \offset]
    lsr     D_h, D_h, \bits
.endm

.macro  memcpy_forward_dst_aligned_epilog, label
    sub     count, count, #0x40
    add     dst, dst, 0x40
    cmp     count, #0x40
    add     src, src, 0x40
    b.ge    \label
    cbnz    count, .memcpy_forward_tail_small
.endm

/* dst is 1 byte aligned src is 8 byte aligned */
.memcpy_forward_dst_aligned_1:
    memcpy_forward_load_array          src
    memcpy_forward_store_shift_array   strb, 0, 0x08
    memcpy_forward_store_shift_array   strb, 1, 0x08
    memcpy_forward_store_shift_array   strb, 2, 0x08
    memcpy_forward_store_shift_array   strb, 3, 0x08
    memcpy_forward_store_shift_array   strb, 4, 0x08
    memcpy_forward_store_shift_array   strb, 5, 0x08
    memcpy_forward_store_shift_array   strb, 6, 0x08
    memcpy_forward_store_array         strb, 7

    memcpy_forward_dst_aligned_epilog  .memcpy_forward_dst_aligned_1
    ret

/* dst is 2 byte aligned src is 8 byte aligned */
.memcpy_forward_dst_aligned_2:
    memcpy_forward_load_array          src
    memcpy_forward_store_shift_array   strh, 0, 0x10
    memcpy_forward_store_shift_array   strh, 2, 0x10
    memcpy_forward_store_shift_array   strh, 4, 0x10
    memcpy_forward_store_array         strh, 6

    memcpy_forward_dst_aligned_epilog  .memcpy_forward_dst_aligned_2
    ret

/* dst is 4 byte aligned src is 8 byte aligned */
.memcpy_forward_dst_aligned_4:
    memcpy_forward_load_array          src
    memcpy_forward_store_shift_array   str, 0, 0x20
    memcpy_forward_store_array         str, 4

    memcpy_forward_dst_aligned_epilog  .memcpy_forward_dst_aligned_4
    ret


// Backward version

memcpy_backward:
    add     src, src, count
    add     dst, dst, count

    /* If count >= 64, use the "not short" version */
    cmp     count, #0x40
    b.ge    .memcpy_backward_not_short

/* Count < 64, src and dst alignments are unknown */
.memcpy_backward_tail_small:
    /* If the length is greater than 8, fall through to code that copies 8
       bytes at a time. Otherwise branch to code that copies the last 8
       bytes */
    cmp     count, #0x08
    b.le    .memcpy_backward_tail8

/* Copy 8 byte chunks one byte at a time until there are less than 8 bytes to
   copy. */
.memcpy_backward_tail_by_8:
    sub     count, count, #0x08
    /* Load 8 bytes before storing any bytes. The CPU's pipeline should not
       stall until the first write. */
    ldrb    A_lw, [src, #-0x01]
    ldrb    A_hw, [src, #-0x02]
    ldrb    B_lw, [src, #-0x03]
    ldrb    B_hw, [src, #-0x04]
    ldrb    C_lw, [src, #-0x05]
    ldrb    C_hw, [src, #-0x06]
    ldrb    D_lw, [src, #-0x07]
    ldrb    D_hw, [src, #-0x08]
    /* Store the values */
    strb    A_lw, [dst, #-0x01]
    strb    A_hw, [dst, #-0x02]
    strb    B_lw, [dst, #-0x03]
    strb    B_hw, [dst, #-0x04]
    strb    C_lw, [dst, #-0x05]
    strb    C_hw, [dst, #-0x06]
    strb    D_lw, [dst, #-0x07]
    strb    D_hw, [dst, #-0x08]
    sub     src, src, #0x08
    sub     dst, dst, #0x08
    cmp     count, #0x08
    b.ge    .memcpy_backward_tail_by_8
    cbnz    count, .memcpy_backward_tail8
    ret

/* Count < 64, src and dst alignment >= 8 */
.memcpy_backward_tail_small_aligned:
    /* If the length is greater than 8, fall through to code that copies 8
       bytes at a time. Otherwise branch to code that copies the last 8
       bytes */
    cmp     count, #0x08
    b.le    .memcpy_backward_tail8

/* Copy 8 byte chunks one byte at a time until there are less than 8 bytes to
   copy. */
.memcpy_backward_tail_by_8_aligned:
    sub     count, count, #0x08
    ldr     A_l, [src, #-0x08]
    str     A_l, [dst, #-0x08]
    sub     src, src, #0x08
    sub     dst, dst, #0x08
    cmp     count, #0x08
    b.ge    .memcpy_backward_tail_by_8_aligned
    cbnz    count, .memcpy_backward_tail8
    ret

/* Copy the last <= 8 bytes one byte at a time */
.memcpy_backward_tail8:
    cbz     count, .memcpy_backward_tail8_end
    /* Do a calculated branch into an unrolled loop in _tail8*/
    and     tmp1, count, #0xf
    adr     tmp2, .memcpy_backward__tail8
    mov     tmp3, #0x08
    sub     tmp3, tmp3, tmp1
    add     tmp1, tmp2, tmp3, LSL #0x03
    br      tmp1

/* Unrolled loop. Each unrolled iteration is 8 bytes long so that tail8 can
   branch into the middle */
.memcpy_backward__tail8:
.macro  memcpy_backward_cpy reg
    ldrb    \reg, [src, #-0x01]! 
    strb    \reg, [dst, #-0x01]!
.endm

    memcpy_backward_cpy     A_lw
    memcpy_backward_cpy     A_hw
    memcpy_backward_cpy     B_lw
    memcpy_backward_cpy     B_hw
    memcpy_backward_cpy     C_lw
    memcpy_backward_cpy     C_hw
    memcpy_backward_cpy     D_lw
    memcpy_backward_cpy     D_hw

.memcpy_backward_tail8_end:
    ret

/* Handle count >= 64. If src is not 8-byte aligned, fall through. */
.memcpy_backward_not_short:
    ands    tmp1, src, #0x07
    b.eq    .memcpy_backward_src_aligned_8
    sub     count, count, tmp1

/* Align src to be 8 byte aligned by copying one byte at a time */
.memcpy_backward_load_align_8:
    memcpy_backward_cpy     A_lw
    subs    tmp1, tmp1, #0x01
    b.gt    .memcpy_backward_load_align_8

    cmp     count, #0x40
    b.lt    .memcpy_backward_tail_small

/* src is 8 byte aligned, count >= 64 */
.memcpy_backward_src_aligned_8:
    ands    tmp1, dst, #0x07
    b.ne    .memcpy_backward_dst_not_aligned_8

/* At this point, src and dest are 8 byte aligned and count >= 64 */
.memcpy_backward_both_aligned:
    /* If count >= 128, branch to the "huge" case */
    cmp     count, #0x80
    b.ge    .memcpy_backward_both_aligned_huge

/* Copy 64 aligned bytes */
.memcpy_backward_both_aligned_big:
    sub     count, count, #0x40
    ldp     A_l, A_h, [src, #-0x10]
    ldp     B_l, B_h, [src, #-0x20]
    ldp     C_l, C_h, [src, #-0x30]
    ldp     D_l, D_h, [src, #-0x40]
    sub     src, src, #0x40
    cmp     count, #0x40

    stp     A_l, A_h, [dst, #-0x10]
    stp     B_l, B_h, [dst, #-0x20]
    stp     C_l, C_h, [dst, #-0x30]
    stp     D_l, D_h, [dst, #-0x40]
    sub     dst, dst, #0x40
    b.ne    .memcpy_backward_tail_small_aligned
    ret

/* count */
.memcpy_backward_both_aligned_huge:
    /* Prime the copy pipeline by loading values into 8 registers. */
    ldp     A_l, A_h, [src, #-0x10]
    ldp     B_l, B_h, [src, #-0x20]
    ldp     C_l, C_h, [src, #-0x30]
    ldp     D_l, D_h, [src, #-0x40]!
    subs    count, count, #0x40
1:
    /* Immediatly after storing the register, load a new value into it. This
       maximizes the number instructions between loading the value and
       storing it. The goal of that is to avoid load stalls */
    stp     A_l, A_h, [dst, #-0x10]
    ldp     A_l, A_h, [src, #-0x10]
    stp     B_l, B_h, [dst, #-0x20]
    ldp     B_l, B_h, [src, #-0x20]
    subs    count, count, #0x40
    stp     C_l, C_h, [dst, #-0x30]
    ldp     C_l, C_h, [src, #-0x30]
    stp     D_l, D_h, [dst, #-0x40]!
    ldp     D_l, D_h, [src, #-0x40]!

    cmp     count, #0x40
    b.ge    1b

    /* Drain the copy pipeline by storing the values in the 8 registers */
    stp     A_l, A_h, [dst, #-0x10]
    stp     B_l, B_h, [dst, #-0x20]
    stp     C_l, C_h, [dst, #-0x30]
    stp     D_l, D_h, [dst, #-0x40]
    sub     dst, dst, #0x40
    cmp     count, #0x00
    b.ne    .memcpy_backward_tail_small_aligned
    ret

/* These blocks of code handle the case where src is >= 8 byte aligned but dst
   is not. Prevent non-naturally aligned stores by manually shifting the data
   before storing it. */
.memcpy_backward_dst_not_aligned_8:
    cmp     tmp1, #0x04
    b.eq    .memcpy_backward_dst_aligned_4
    tst     tmp1, #0x01
    b.eq    .memcpy_backward_dst_aligned_2

/* Load 64 bytes into 8 byte registers */
.macro  memcpy_backward_load_array, src
    ldp     A_h, A_l, [\src, #-0x10]
    ldp     B_h, B_l, [\src, #-0x20]
    ldp     C_h, C_l, [\src, #-0x30]
    ldp     D_h, D_l, [\src, #-0x40]
.endm

/* Store a value from each register in the array */
.macro  memcpy_backward_store_array, str, offset
    \str    A_lw, [dst, #-0x08 + \offset]
    \str    A_hw, [dst, #-0x10 + \offset]
    \str    B_lw, [dst, #-0x18 + \offset]
    \str    B_hw, [dst, #-0x20 + \offset]
    \str    C_lw, [dst, #-0x28 + \offset]
    \str    C_hw, [dst, #-0x30 + \offset]
    \str    D_lw, [dst, #-0x38 + \offset]
    \str    D_hw, [dst, #-0x40 + \offset]
.endm

/* Store the lower bits in the registers and then shift the data right to get
   new data in the lower bits. */
.macro  memcpy_backward_store_shift_array, str, offset, bits
    \str    A_lw, [dst, #-0x08 + \offset]
    lsr     A_l, A_l, \bits
    \str    A_hw, [dst, #-0x10 + \offset]
    lsr     A_h, A_h, \bits
    \str    B_lw, [dst, #-0x18 + \offset]
    lsr     B_l, B_l, \bits
    \str    B_hw, [dst, #-0x20 + \offset]
    lsr     B_h, B_h, \bits
    \str    C_lw, [dst, #-0x28 + \offset]
    lsr     C_l, C_l, \bits
    \str    C_hw, [dst, #-0x30 + \offset]
    lsr     C_h, C_h, \bits
    \str    D_lw, [dst, #-0x38 + \offset]
    lsr     D_l, D_l, \bits
    \str    D_hw, [dst, #-0x40 + \offset]
    lsr     D_h, D_h, \bits
.endm

.macro  memcpy_backward_dst_aligned_epilog, label
    sub     count, count, #0x40
    sub     dst, dst, 0x40
    cmp     count, #0x40
    sub     src, src, 0x40
    b.ge    \label
    cbnz    count, .memcpy_backward_tail_small
.endm

/* dst is 1 byte aligned src is 8 byte aligned */
.memcpy_backward_dst_aligned_1:
    memcpy_backward_load_array          src
    memcpy_backward_store_shift_array   strb, 0, 0x08
    memcpy_backward_store_shift_array   strb, 1, 0x08
    memcpy_backward_store_shift_array   strb, 2, 0x08
    memcpy_backward_store_shift_array   strb, 3, 0x08
    memcpy_backward_store_shift_array   strb, 4, 0x08
    memcpy_backward_store_shift_array   strb, 5, 0x08
    memcpy_backward_store_shift_array   strb, 6, 0x08
    memcpy_backward_store_array         strb, 7

    memcpy_backward_dst_aligned_epilog  .memcpy_backward_dst_aligned_1
    ret

/* dst is 2 byte aligned src is 8 byte aligned */
.memcpy_backward_dst_aligned_2:
    memcpy_backward_load_array          src
    memcpy_backward_store_shift_array   strh, 0, 0x10
    memcpy_backward_store_shift_array   strh, 2, 0x10
    memcpy_backward_store_shift_array   strh, 4, 0x10
    memcpy_backward_store_array         strh, 6

    memcpy_backward_dst_aligned_epilog  .memcpy_backward_dst_aligned_2
    ret

/* dst is 4 byte aligned src is 8 byte aligned */
.memcpy_backward_dst_aligned_4:
    memcpy_backward_load_array          src
    memcpy_backward_store_shift_array   str, 0, 0x20
    memcpy_backward_store_array         str, 4

    memcpy_backward_dst_aligned_epilog  .memcpy_backward_dst_aligned_4
    ret
