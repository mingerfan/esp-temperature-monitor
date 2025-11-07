pub fn quick_align(val: usize, align: usize) -> usize {
    if align == 0 {
        return val;
    }
    if (align & (align - 1)) == 0 {
        // align 是 2 的幂次方 (使用位运算，最快)
        (val + align - 1) & !(align - 1)
    } else {
        // align 不是 2 的幂次方 (使用通用模数运算)

        val.div_ceil(align) * align
    }
}