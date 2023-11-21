use pretty_assertions::assert_eq;
use rstest::*;
use ps2_floating_point::Ps2Float;

#[rstest]
#[case(0x40A9999A)] // 5.3
#[case(0x00000000)] // 0.00
#[case(0x7FFFFFFF)] // MAX
#[case(0xFFFFFFFF)] // -MAX
#[case(0x7F800000)] // INF
#[case(0xFF800000)] // -INF
fn ps2float_new(#[case] value: u32) {
    let ps2float = Ps2Float::new(value);

    assert_eq!(ps2float.as_u32(), value);
}

#[rstest]
#[case(false, 0x81, 0x29999A, 0x40A9999A)] // 5.3
#[case(false, 0, 0, 0)] // 0.00
#[case(false, 0xFF, 0x7FFFFF, 0x7FFFFFFF)] // MAX
#[case(true, 0xFF, 0x7FFFFF, 0xFFFFFFFF)] // -MAX
#[case(false, 0xFF, 0, 0x7F800000)] // INF
#[case(true, 0xFF, 0, 0xFF800000)] // -INF
fn ps2float_from_params(
    #[case] sign: bool,
    #[case] exponent: u8,
    #[case] mantissa: u32,
    #[case] expected: u32,
) {
    let ps2float = Ps2Float::from_params(sign, exponent, mantissa);

    assert_eq!(ps2float.as_u32(), expected);
}

// Adapted from unknownbrackets/ps2autotests FPU arithmetic test: https://bit.ly/3sdgA6g
#[rstest]
#[case(0x00000000, 0x00000000, 0x00000000)] // 0.00 + 0.00 = 0.00
#[case(0x00000000, 0x80000000, 0x00000000)] // 0.00 + -0.00 = 0.00
#[case(0x80000000, 0x00000000, 0x00000000)] // -0.00 + 0.00 = 0.00
#[case(0x80000000, 0x80000000, 0x80000000)] // -0.00 + -0.00 = -0.00
#[case(0x00000000, 0x3F800000, 0x3F800000)] // 0.00 + 1.00 = 1.00
#[case(0x3F800000, 0x3F800000, 0x40000000)] // 1.00 + 1.00 = 2.00
#[case(0x3F800000, 0x00000000, 0x3F800000)] // 1.00 + 0.00 = 1.00
#[case(0x40000000, 0x40000000, 0x40800000)] // 2.00 + 2.00 = 4.00
#[case(0x40400000, 0x3F800000, 0x40800000)] // 3.00 + 1.00 = 4.00
#[case(0x40400000, 0x40400000, 0x40C00000)] // 3.00 + 3.00 = 6.00
#[case(0x7FFFFFFF, 0x7FFFFFFF, 0x7FFFFFFF)] // MAX + MAX = MAX
#[case(0x7FFFFFFF, 0xFFFFFFFF, 0x00000000)] // MAX + -MAX = 0.00
#[case(0xFFFFFFFF, 0x7FFFFFFF, 0x7FFFFFFF)] // -MAX + MAX = MAX
#[case(0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF)] // -MAX + -MAX = -MAX
#[case(0x7FFFFFFF, 0x00000000, 0x7FFFFFFF)] // MAX + 0.00 = MAX
#[case(0x00000000, 0x7FFFFFFF, 0x7FFFFFFF)] // 0.00 + MAX = MAX
#[case(0x00000000, 0x7F800000, 0x7F800000)] // 0.00 + INF = INF
#[case(0x7F800000, 0x7F800000, 0x7FFFFFFF)] // INF + INF = MAX
#[case(0xFF800000, 0x7F800000, 0x00000000)] // -INF + INF = 0.00
fn ps2float_add(#[case] a_addend: u32, #[case] b_addend: u32, #[case] expected: u32) {
    let a = Ps2Float::new(a_addend);
    let b = Ps2Float::new(b_addend);

    let result = a.add(&b);

    assert_eq!(
        result.as_u32(),
        expected,
        "Testing adding floats {} and {} == {:X}",
        a_addend,
        b_addend,
        expected
    );
}

// Adapted from unknownbrackets/ps2autotests FPU subtraction test: https://bit.ly/3YJxp52
#[rstest]
#[case(0x00000000, 0x00000000, 0x00000000)] // 0.00 - 0.00 = 0.00
#[case(0x00000000, 0x80000000, 0x00000000)] // 0.00 - -0.00 = 0.00
#[case(0x80000000, 0x00000000, 0x80000000)] // -0.00 - 0.00 = -0.00
#[case(0x80000000, 0x80000000, 0x00000000)] // -0.00 - -0.00 = 0.00
#[case(0x00000000, 0x3F800000, 0xBF800000)] // 0.00 - 1.00 = -1.00
#[case(0x3F800000, 0x3F800000, 0x00000000)] // 1.00 - 1.00 = 0.00
#[case(0x3F800000, 0x00000000, 0x3F800000)] // 1.00 - 0.00 = 1.00
#[case(0x40000000, 0x40000000, 0x00000000)] // 2.00 - 2.00 = 0.00
#[case(0x40400000, 0x3F800000, 0x40000000)] // 3.00 - 1.00 = 2.00
#[case(0x40400000, 0x40400000, 0x00000000)] // 3.00 - 3.00 = 0.00
#[case(0x7FFFFFFF, 0x7FFFFFFF, 0x00000000)] // MAX - MAX = 0.00
#[case(0x7FFFFFFF, 0xFFFFFFFF, 0x7FFFFFFF)] // MAX - -MAX = MAX
#[case(0xFFFFFFFF, 0x7FFFFFFF, 0xFFFFFFFF)] // -MAX - MAX = -MAX
#[case(0xFFFFFFFF, 0xFFFFFFFF, 0x00000000)] // -MAX - -MAX = 0.00
#[case(0x7FFFFFFF, 0x00000000, 0x7FFFFFFF)] // MAX - 0.00 = MAX
#[case(0x00000000, 0x7FFFFFFF, 0xFFFFFFFF)] // 0.00 - MAX = -MAX
#[case(0x00000000, 0x7F800000, 0xFF800000)] // 0.00 - INF = -INF
#[case(0x7F800000, 0x7F800000, 0x00000000)] // INF - INF = 0.00
#[case(0xFF800000, 0x7F800000, 0xFFFFFFFF)] // -INF - INF = -MAX
fn ps2float_sub(#[case] a_subtrahend: u32, #[case] b_subtrahend: u32, #[case] expected: u32) {
    let a = Ps2Float::new(a_subtrahend);
    let b = Ps2Float::new(b_subtrahend);

    let result = a.sub(&b);

    assert_eq!(
        result.as_u32(),
        expected,
        "Testing subtracting floats {} and {} == {:X}",
        a_subtrahend,
        b_subtrahend,
        expected
    );
}
