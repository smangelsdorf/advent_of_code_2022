fn to_snafu(mut n: i64) -> String {
    // Worst case is like... 28?
    let mut digits = Vec::with_capacity(32);

    while n > 0 {
        let digit = match n.rem_euclid(5) {
            0 => b'0',
            1 => b'1',
            2 => b'2',
            3 => b'=',
            4 => b'-',
            _ => panic!("modulus not in range"),
        };

        n = n / 5 + (if digit.is_ascii_digit() { 0 } else { 1 });
        digits.push(digit);
    }

    digits.reverse();
    String::from_utf8(digits).expect("valid utf8")
}

fn from_snafu(s: &str) -> Result<i64, (&'static str, u8)> {
    s.bytes().try_fold(0, |mut acc, b| {
        acc *= 5;

        match b {
            b'0' => Ok(acc),
            b'1' => Ok(acc + 1),
            b'2' => Ok(acc + 2),
            b'-' => Ok(acc - 1),
            b'=' => Ok(acc - 2),
            _ => Err(("unrecognised digit", b)),
        }
    })
}

pub fn main() {
    let total = std::io::stdin()
        .lines()
        .map(|r| from_snafu(&r.unwrap()).unwrap())
        .sum::<i64>();

    println!("Decimal total: {}", total);
    println!("SNAFU total: {}", to_snafu(total));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snafu_convert() {
        for i in 0..100_000 {
            let s = to_snafu(i);
            let n = from_snafu(&s).unwrap();

            assert_eq!(n, i);
        }

        let expected = [
            (1, "1"),
            (2, "2"),
            (3, "1="),
            (4, "1-"),
            (5, "10"),
            (6, "11"),
            (7, "12"),
            (8, "2="),
            (9, "2-"),
            (10, "20"),
            (15, "1=0"),
            (20, "1-0"),
            (2022, "1=11-2"),
            (12345, "1-0---0"),
            (314159265, "1121-1110-1=0"),
        ];

        for (n, s) in expected {
            assert_eq!(to_snafu(n), s);
            assert_eq!(from_snafu(s).unwrap(), n);
        }
    }
}
