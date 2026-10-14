use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Cipher {
    Ordinal,
    Reduction,
    Reverse,
    ReverseReduction,
    Standard,
    ReverseStandard,
    Sumerian,
    ReverseSumerian,
    Primes,
    ReversePrimes,
    Squares,
    Trigonal,
    Septenary,
}
impl Cipher {
    pub const ALL: [Self; 13] = [
        Self::Ordinal,
        Self::Reduction,
        Self::Reverse,
        Self::ReverseReduction,
        Self::Standard,
        Self::ReverseStandard,
        Self::Sumerian,
        Self::ReverseSumerian,
        Self::Primes,
        Self::ReversePrimes,
        Self::Squares,
        Self::Trigonal,
        Self::Septenary,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Self::Ordinal => "Ordinal",
            Self::Reduction => "Reduction",
            Self::Reverse => "Reverse Ordinal",
            Self::ReverseReduction => "Reverse Reduction",
            Self::Standard => "Standard",
            Self::ReverseStandard => "Reverse Standard",
            Self::Sumerian => "Sumerian",
            Self::ReverseSumerian => "Reverse Sumerian",
            Self::Primes => "Primes",
            Self::ReversePrimes => "Reverse Primes",
            Self::Squares => "Squares",
            Self::Trigonal => "Trigonal",
            Self::Septenary => "Septenary",
        }
    }
    pub fn weight(self, c: char) -> i64 {
        if !c.is_ascii_alphabetic() {
            return 0;
        }
        let n = i64::from(c.to_ascii_uppercase() as u8 - b'A') + 1;
        let standard = |i: i64| {
            if i <= 9 {
                i
            } else if i <= 18 {
                (i - 9) * 10
            } else {
                (i - 18) * 100
            }
        };
        let primes = [
            2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71, 73, 79, 83,
            89, 97, 101,
        ];
        match self {
            Self::Ordinal => n,
            Self::Reduction => (n - 1) % 9 + 1,
            Self::Reverse => 27 - n,
            Self::ReverseReduction => (26 - n) % 9 + 1,
            Self::Standard => standard(n),
            Self::ReverseStandard => standard(27 - n),
            Self::Sumerian => n * 6,
            Self::ReverseSumerian => (27 - n) * 6,
            Self::Primes => primes[(n - 1) as usize],
            Self::ReversePrimes => primes[(26 - n) as usize],
            Self::Squares => n * n,
            Self::Trigonal => n * (n + 1) / 2,
            Self::Septenary => [1, 2, 3, 4, 5, 6, 7, 6, 5, 4, 3, 2, 1][((n - 1) % 13) as usize],
        }
    }
    pub fn total(self, text: &str) -> i64 {
        text.chars().map(|c| self.weight(c)).sum()
    }
    pub fn breakdown(self, text: &str) -> String {
        text.chars()
            .filter(|c| c.is_ascii_alphabetic())
            .map(|c| format!("{}:{}", c, self.weight(c)))
            .collect::<Vec<_>>()
            .join(" + ")
    }
}
