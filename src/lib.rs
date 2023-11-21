//! PS2 IEEE 754 floating-point variant number implementation.

use std::cmp::Ordering;
use std::fmt::{Debug, Display, Formatter};

/// A floating point number in the PS2's IEEE 754 variant format.
///
/// See: https://www.gregorygaines.com/blog/emulating-ps2-floating-point-nums-ieee-754-diffs-part-1/
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub struct Ps2Float {
    sign: bool,
    exponent: u8,
    mantissa: u32,
}

/// Creating a new PS2 float.
impl Ps2Float {
    /// The maximum possible value for an PS2 IEEE 754 variant float.
    const MAX_FLOATING_POINT_VALUE: u32 = 0x7FFFFFFF;

    /// The minimum possible value for an PS2 IEEE 754 variant float.
    const MIN_FLOATING_POINT_VALUE: u32 = 0xFFFFFFFF;

    /// Creates a new PS2 float from the given value.
    ///
    /// # Arguments
    ///
    /// * `value` - The decimal value of the float.
    ///
    /// # Returns
    ///
    /// A PS2 IEEE 754 variant float.
    pub fn new(value: u32) -> Self {
        Self {
            sign: ((value >> 31) & 1) != 0,
            exponent: ((value >> 23) & 0xFF) as u8,
            mantissa: value & 0x7FFFFF,
        }
    }

    /// Creates a new PS2 float from the given float parameters.
    ///
    /// # Arguments
    ///
    /// * `sign` - The sign of the float.
    /// * `exponent` - The exponent of the float.
    /// * `mantissa` - The mantissa of the float.
    ///
    /// # Returns
    ///
    /// A PS2 IEEE 754 variant float.
    pub fn from_params(sign: bool, exponent: u8, mantissa: u32) -> Self {
        Self { sign, exponent, mantissa }
    }

    /// Returns a [`PS2Float`] float representing the maximum possible value of
    /// an PS2 IEEE 754 float.
    ///
    /// The maximum possible value also goes by as MAX, Fmax, NaN.
    pub fn max() -> Self {
        Self::new(Self::MAX_FLOATING_POINT_VALUE)
    }

    /// Returns a [`PS2Float`] float representing the minimum possible value of
    /// an PS2 IEEE 754 float.
    ///
    /// The minimum possible value also goes by as -MAX, -Fmax, NaN, or -NaN.
    pub fn min() -> Self {
        Self::new(Self::MIN_FLOATING_POINT_VALUE)
    }

    /// Returns the decimal representation of `self`.
    pub fn as_u32(&self) -> u32 {
        let mut result = 0u32;
        result |= (self.sign as u32) << 31;
        result |= (self.exponent as u32) << 23;
        result |= self.mantissa;
        result
    }
}

/// Implementing adding and subtracting arithmetic operations on PS2 floats.
impl Ps2Float {
    /// The bit position of the implicit leading bit in the mantissa.
    const IMPLICIT_LEADING_BIT_POS: i32 = 23;

    /// The positive infinity value of a PS2 IEEE 754 variant float.
    const POSITIVE_INFINITY_VALUE: u32 = 0x7F800000;

    /// The negative infinity value of a PS2 IEEE 754 variant float.
    const NEGATIVE_INFINITY_VALUE: u32 = 0xFF800000;

    /// Adds two PS2 floats together.
    ///
    /// See: TODO article part 2
    ///
    /// # Arguments
    ///
    /// * `addend` - The addend float to add to `self`.
    ///
    /// # Returns
    ///
    /// A PS2 IEEE 754 variant float representing the sum of the two floats.
    pub fn add(&self, addend: &Ps2Float) -> Self {
        // Check if either number is denormalized because denormalized floats don't
        // exist on the PS2 and truncated to zero during arithmetic operations.
        if self.is_denormalized() || addend.is_denormalized() {
            return Self::solve_demoralized_operation(self, addend, /* add= */ true);
        }

        // Check if abnormal operation between two NaN or Inf number.
        if self.is_abnormal() && addend.is_abnormal() {
            return Self::solve_abnormal_addition_or_subtraction_operation(
                self, addend, /* add= */ true,
            );
        }

        // Only add floats with the same sign, otherwise subtract.
        if self.sign != addend.sign {
            return self.sub(addend);
        }

        self.do_add_or_sub(addend, /* add= */ true)
    }

    /// Subtracts two PS2 floats from each other.
    ///
    /// See: TODO article part 2
    ///
    /// # Arguments
    ///
    /// * `subtrahend` - The addend float to subtract from `self`.
    ///
    /// # Returns
    ///
    /// A PS2 IEEE 754 variant float representing the difference between the two
    /// floats.
    pub fn sub(&self, subtrahend: &Ps2Float) -> Self {
        // Check if either number is denormalized because denormalized floats don't
        // exist on the PS2 and truncated to zero during arithmetic operations.
        if self.is_denormalized() || subtrahend.is_denormalized() {
            return Self::solve_demoralized_operation(self, subtrahend, /* add= */ false);
        }

        // Check if abnormal operation between two NaN or Inf number.
        if self.is_abnormal() && subtrahend.is_abnormal() {
            return Self::solve_abnormal_addition_or_subtraction_operation(
                self, subtrahend, /* add= */ false,
            );
        }

        // Check if both numbers are equal, if so the result is zero.
        if self.cmp(subtrahend) == Ordering::Equal {
            let mut result = Self::new(0);
            result.sign = Self::determine_subtraction_operation_sign(self, subtrahend);
            return result;
        }

        self.do_add_or_sub(subtrahend, /* add= */ false)
    }

    /// Solves an addition or subtraction operation between two abnormal floats.
    fn solve_abnormal_addition_or_subtraction_operation(
        a: &Ps2Float,
        b: &Ps2Float,
        add: bool,
    ) -> Ps2Float {
        let a_val = a.as_u32();
        let b_val = b.as_u32();

        if a_val == Self::MAX_FLOATING_POINT_VALUE && b_val == Self::MAX_FLOATING_POINT_VALUE {
            // MAX + MAX = MAX
            return if add {
                Self::max()
            } else {
                // MAX - MAX = 0
                Self::default()
            };
        }

        if a_val == Self::MIN_FLOATING_POINT_VALUE && b_val == Self::MIN_FLOATING_POINT_VALUE {
            // -MIN + -MIN = MAX
            return if add {
                Self::min()
            } else {
                // -MIN - -MIN = 0
                Self::default()
            };
        }

        if a_val == Self::MIN_FLOATING_POINT_VALUE && b_val == Self::MAX_FLOATING_POINT_VALUE {
            // -MAX + MAX = MAX
            return if add {
                Self::max()
            } else {
                // -MAX - MAX = MIN
                Self::min()
            };
        }

        if a_val == Self::MAX_FLOATING_POINT_VALUE && b_val == Self::MIN_FLOATING_POINT_VALUE {
            // MAX + -MAX = 0
            return if add {
                Self::default()
            } else {
                // MAX - -MAX = MIN
                Self::max()
            };
        }

        if a_val == Self::POSITIVE_INFINITY_VALUE && b_val == Self::POSITIVE_INFINITY_VALUE {
            // INF + INF = MAX
            return if add {
                Self::max()
            } else {
                // INF - INF = 0
                Self::default()
            };
        }

        if a_val == Self::NEGATIVE_INFINITY_VALUE && b_val == Self::POSITIVE_INFINITY_VALUE {
            // -INF + INF = 0
            return if add {
                Self::default()
            } else {
                // -INF - INF = -MAX
                Self::min()
            };
        }

        if a_val == Self::NEGATIVE_INFINITY_VALUE && b_val == Self::NEGATIVE_INFINITY_VALUE {
            // -INF + -INF = min
            return if add {
                Self::min()
            } else {
                // -INF - -INF = 0
                Self::default()
            };
        }

        panic!("Unhandled abnormal floating point operation");
    }

    /// Internal implementation of adding or subtracts two PS2 floats.
    ///
    /// The addition and subtraction algorithms are mostly the same so they can
    /// be condensed into one function and called from the public API.
    ///
    /// See: TODO article part 2
    ///
    /// # Arguments
    ///
    /// * `other` - The other float to add or subtract.
    /// * `add` - Adds if true, otherwise subtract.
    ///
    /// # Returns
    ///
    /// A [`Ps2Float`] representing the sum or difference between two floats.
    fn do_add_or_sub(&self, other: &Ps2Float, add: bool) -> Ps2Float {
        // Find the absolute value of the exponent difference.
        let exp_diff = self.exponent.abs_diff(other.exponent);

        // Add implicit leading bit to both mantissa.
        let mut self_mantissa = self.mantissa | 0x800000;
        let mut other_mantissa = other.mantissa | 0x800000;

        let mut result = Self::default();

        // Align the exponents.
        if self.exponent >= other.exponent {
            other_mantissa = other_mantissa.wrapping_shr(exp_diff as u32);
            result.exponent = self.exponent;
        } else {
            self_mantissa = self_mantissa.wrapping_shr(exp_diff as u32);
            result.exponent = other.exponent;
        }

        if add {
            result.mantissa = self_mantissa.wrapping_add(other_mantissa);
            // Both numbers have the same sign.
            result.sign = self.sign;
        } else {
            // Subtract
            result.mantissa = self_mantissa.wrapping_sub(other_mantissa);
            // Take the sign of the bigger mantissa.
            result.sign = Self::determine_subtraction_operation_sign(self, other);
        }

        // Normalize the result if needed.
        if result.mantissa > 0 {
            let mut leading_bit_position = Self::get_most_significant_bit_position(result.mantissa);
            while leading_bit_position != Self::IMPLICIT_LEADING_BIT_POS {
                match leading_bit_position.cmp(&Self::IMPLICIT_LEADING_BIT_POS) {
                    Ordering::Greater => {
                        result.mantissa = result.mantissa.wrapping_shr(1);

                        // Check for exponent overflow, if so return +/- max value depending on the
                        // sign.
                        let checked_exponent_increment = result.exponent.checked_add(1);
                        match checked_exponent_increment {
                            None => {
                                return if result.sign { Self::min() } else { Self::max() };
                            }
                            Some(res) => result.exponent = res,
                        }
                        leading_bit_position -= 1;
                    }
                    Ordering::Less => {
                        result.mantissa = result.mantissa.wrapping_shl(1);

                        // Check for exponent underflow, if so the result is a denormalized float
                        // which doesn't exist so return +/- 0 depending on
                        // the sign.
                        let checked_exponent_decrement = result.exponent.checked_sub(1);
                        match checked_exponent_decrement {
                            None => return Self::from_params(result.sign, 0, 0),
                            Some(res) => result.exponent = res,
                        }
                        leading_bit_position += 1;
                    }
                    Ordering::Equal => {}
                }
            }
        }

        // Remove implicit leading bit from mantissa.
        result.mantissa &= 0x7FFFFF;

        result.round_towards_zero()
    }

    /// Solves an addition or subtraction operation between two denormalized
    /// floats.
    fn solve_demoralized_operation(a: &Ps2Float, b: &Ps2Float, add: bool) -> Ps2Float {
        let mut result;
        if a.is_denormalized() && !b.is_denormalized() {
            result = *b;
        } else if !a.is_denormalized() && b.is_denormalized() {
            result = *a;
        } else if a.is_denormalized() && b.is_denormalized() {
            result = Self::default();
        } else {
            panic!("Both numbers are not denormalized");
        }

        if add {
            result.sign = Self::determine_addition_operation_sign(a, b);
        } else {
            result.sign = Self::determine_subtraction_operation_sign(a, b);
        }

        result
    }

    /// Determines the sign of an addition operation.
    fn determine_addition_operation_sign(a: &Ps2Float, b: &Ps2Float) -> bool {
        if a.is_zero() && b.is_zero() {
            if !a.sign || !b.sign {
                return false;
            } else if a.sign && b.sign {
                return true;
            } else {
                panic!("Unhandled addition operation flags");
            }
        }

        a.sign
    }

    /// Determines the sign of an subtraction operation.
    fn determine_subtraction_operation_sign(a: &Ps2Float, b: &Ps2Float) -> bool {
        if a.is_zero() && b.is_zero() {
            if !a.sign || b.sign {
                return false;
            } else if a.sign && !b.sign {
                return true;
            } else {
                panic!("Unhandled subtraction operation flags");
            }
        }

        // Flip the sign of the second number aka Keep change change.
        let b_sign = !b.sign;

        match a.cmp(b) {
            Ordering::Less => b_sign,
            Ordering::Equal => a.sign,
            Ordering::Greater => a.sign,
        }
    }

    /// Returns the place of the leading set bit in the given value.
    fn get_most_significant_bit_position(value: u32) -> i32 {
        let mut bit = 31;

        while bit >= 0 {
            if ((value >> bit) & 1) != 0 {
                return bit;
            }
            bit -= 1;
        }

        bit
    }

    /// Returns if the float is denormalized.
    ///
    /// Denormalized floats have an exponent of 0.
    fn is_denormalized(&self) -> bool {
        self.exponent == 0
    }

    /// Returns if the float has an abnormal value.
    ///
    /// Abnormal values numbers that or +/- Infinity or NaN aka Fmax/Fmax.
    fn is_abnormal(&self) -> bool {
        let val = self.as_u32();

        val == Self::MAX_FLOATING_POINT_VALUE
            || val == Self::MIN_FLOATING_POINT_VALUE
            || val == Self::POSITIVE_INFINITY_VALUE
            || val == Self::NEGATIVE_INFINITY_VALUE
    }

    /// Returns if the float is zero.
    ///
    /// Checks if the absolute value of the float is zero.
    fn is_zero(&self) -> bool {
        self.as_u32() & 0x7FFFFFFF == 0
    }

    /// Returns only the integer part of the float.
    ///
    /// Everything after the decimal point is discarded.
    fn round_towards_zero(&self) -> Ps2Float {
        let mut ps2_float_double = self.as_u32() as f64;
        ps2_float_double = ps2_float_double.trunc();
        Self::new(ps2_float_double as u32)
    }
}

/// Implementing multiplying and dividing arithmetic operations on PS2 floats.
impl Ps2Float {
    pub fn mul(&self, _factor: &Ps2Float) -> Ps2Float {
        unimplemented!("TODO Add multiplication implementation")
    }

    pub fn div(&self, _factor: &Ps2Float) -> Ps2Float {
        unimplemented!("TODO Add division implementation")
    }
}

impl Display for Ps2Float {
    /// Formats the float as a string.
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let exponent = self.exponent as f64 - 127.;
        let mantissa = self.mantissa as f64 / 2_f64.powf(23.) + 1.0;

        let mut res = mantissa * 2f64.powf(exponent);
        if self.sign {
            res *= -1.0;
        }

        let value = self.as_u32();
        if self.is_denormalized() {
            return write!(f, "Denormalized({:.2})", res);
        } else if value == Self::MAX_FLOATING_POINT_VALUE {
            return write!(f, "Fmax({:.2})", res);
        } else if value == Self::MIN_FLOATING_POINT_VALUE {
            return write!(f, "-Fmax({:.2})", res);
        } else if value == Self::POSITIVE_INFINITY_VALUE {
            return write!(f, "Inf({:.2})", res);
        } else if value == Self::NEGATIVE_INFINITY_VALUE {
            return write!(f, "-Inf({:.2})", res);
        }

        write!(f, "{:.2}", res)
    }
}

impl PartialOrd<Self> for Ps2Float {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Ps2Float {
    /// Compares two floats and returns the [`Ordering`] between `self` and
    /// `other`.
    ///
    /// The comparison is done comparing both floats by their 2's complement
    /// representation.
    fn cmp(&self, other: &Self) -> Ordering {
        let mut self_two_complement_val = (self.as_u32() & 0x7FFFFFFF) as i32;
        if self.sign {
            self_two_complement_val = -self_two_complement_val;
        }

        let mut other_two_complement_val = (other.as_u32() & 0x7FFFFFFF) as i32;
        if other.sign {
            other_two_complement_val = -other_two_complement_val;
        }

        self_two_complement_val.cmp(&other_two_complement_val)
    }
}
