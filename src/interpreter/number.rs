use anyhow::{Result, anyhow, bail};
use rug::{Integer, Rational};

// Truncated-toward-zero modulo on rationals: a - b * trunc(a / b).
pub(super) fn rat_mod(a: &Rational, b: &Rational) -> Rational {
    let q = Rational::from(a / b);
    let (num, den) = q.into_numer_denom();
    let trunc = Integer::from(&num / &den);
    a - Rational::from(b * trunc)
}

// `**` requires an integer exponent; non-integer exponents would produce
// irrationals that don't fit in Rational.
pub(super) fn rat_pow(base: &Rational, exp: &Rational) -> Result<Rational> {
    if exp.denom() != &Integer::from(1) {
        bail!(
            "** requires an integer exponent, got {}",
            format_rational(exp)
        );
    }
    let e_int = exp.numer();
    let e: i32 = e_int
        .to_i32()
        .ok_or_else(|| anyhow!("** exponent out of range: {}", e_int))?;
    if e == 0 {
        return Ok(Rational::from(1));
    }
    if base.cmp0() == std::cmp::Ordering::Equal && e < 0 {
        bail!("0 cannot be raised to a negative power");
    }
    use rug::ops::Pow;
    Ok(base.clone().pow(e))
}

// Display a rational as the most natural decimal form:
// - integer when denom == 1
// - terminating decimal (e.g. 5/2 -> "2.5") when denom is 2^a * 5^b
// - otherwise the canonical "n/d" form
pub(super) fn format_rational(r: &Rational) -> String {
    let num = r.numer();
    let den = r.denom();
    if den == &Integer::from(1) {
        return num.to_string();
    }

    let mut d = den.clone();
    let mut twos: u32 = 0;
    while d.is_divisible_u(2) {
        d /= 2u32;
        twos += 1;
    }
    let mut fives: u32 = 0;
    while d.is_divisible_u(5) {
        d /= 5u32;
        fives += 1;
    }
    if d != 1 {
        return format!("{}/{}", num, den);
    }

    let power = twos.max(fives);
    let mut scaled = num.clone().abs();
    let mut extra_twos = power - twos;
    while extra_twos > 0 {
        scaled *= 2u32;
        extra_twos -= 1;
    }
    let mut extra_fives = power - fives;
    while extra_fives > 0 {
        scaled *= 5u32;
        extra_fives -= 1;
    }

    let sign = if num.cmp0() == std::cmp::Ordering::Less {
        "-"
    } else {
        ""
    };
    let digits = scaled.to_string();
    let p = power as usize;
    let padded = if digits.len() <= p {
        let pad = "0".repeat(p - digits.len() + 1);
        format!("{}{}", pad, digits)
    } else {
        digits
    };
    let split = padded.len() - p;
    format!("{}{}.{}", sign, &padded[..split], &padded[split..])
}

pub(super) fn rat_succ(r: &Rational) -> Rational {
    let mut out = r.clone();
    out += 1;
    out
}

pub(super) fn range_has_elem(cur: &Rational, end: Option<&Rational>, inclusive: bool) -> bool {
    match end {
        None => true,
        Some(e) => {
            if inclusive {
                cur <= e
            } else {
                cur < e
            }
        }
    }
}
