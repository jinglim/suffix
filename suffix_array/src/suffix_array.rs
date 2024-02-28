// Builds a SuffixArray.

#[allow(dead_code)]
const DEBUG_LEVEL: usize = 1;

// Assuming the size of text is less than 2^TextSize.
pub type TextSize = u32;

pub trait SuffixArrayBuilder {
    // Builds a suffix array from a given text.
    fn build(&self, text: &[u8]) -> Box<dyn SuffixArray>;
}

// SuffixArray contains a list of ranked positions of suffix strings.
// e.g. the suffix array of "cat" is [1, 0, 2].
pub trait SuffixArray {
    // Returns an iterator.
    fn iter<'a>(&'a self) -> Box<dyn Iterator<Item = TextSize> + 'a>;

    // Returns an array.
    fn array(&self) -> &[TextSize];
}

// Validates a suffix array is correct.
pub fn validate_suffix_array(text: &[u8], sa: &[TextSize]) {
    // Make an inverse suffix array so that inverse_sa[sa[i]] = i.
    let len = text.len() as TextSize;
    let mut inverse_sa: Vec<TextSize> = vec![0; len as usize];
    for i in 0..len {
        inverse_sa[sa[i as usize] as usize] = i;
    }

    // Start with sa[0].
    let mut prev_ch = text[sa[0] as usize];
    let mut prev_rank : isize = {
        let pos = sa[0];
        if pos == len - 1 {
            -1
        } else {
            inverse_sa[(pos + 1) as usize] as isize
        }
    };

    // For sa[1 .. len-1], check that if the first char of sa[i] == first char of sa[i+1],
    // then the inverse_sa[sa[i] + 1] < inverse_sa[sa[i + 1] + 1].
    // For instance, "abc" < "abd", with common first char "a", check that "bc" < "bd". 
    for i in 1..len - 1 {
        let pos = sa[i as usize];
        let ch = text[pos as usize];
        if ch == prev_ch {
            assert!(pos + 1 < len as TextSize);
            let rank = inverse_sa[(pos + 1) as usize] as isize;
            if rank <= prev_rank {
                println!("At {} pos {}", i, pos);
                panic!("Rank {} should be > {}", rank, prev_rank);
            }
            prev_rank = rank;
        } else {
            assert!(ch > prev_ch);
            prev_ch = ch;
            if pos == len - 1 {
                prev_rank = -1;
            } else {
                prev_rank = inverse_sa[(pos + 1) as usize] as isize;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::testing::testing;
    use super::*;

    // Test building suffix array from some short strings.
    fn test_short_strings(sa_builder: &dyn SuffixArrayBuilder) {
        let test_strings = [
            "a",
            "aaaaaaaa",
            "aaaaaaab",
            "baaaaaaa",
            "abaaaaaa",
            "abababab",
            "abcbabcba",
            "cabbage abc food abc vegetables",
            "ktrthleluzsxleo",
            "fjhccfahejdacbffahbf",
            "wprnnuivivdygnarkzkjmvmpuxuzbsehrmunexkvkjczbbrawh",
            "zdzkixqfehvgmrqpevpqmnefcjsppmqxwbibbkylelyjcjulejunynoxypzbcjlw",
        ];
        for test_str in test_strings {
            let test_bytes = test_str.as_bytes();

            if DEBUG_LEVEL >= 1 {
                println!(
                    "Test string: {}",
                    String::from_utf8_lossy(&test_bytes).to_string()
                );
            }

            let suffix_array = sa_builder.build(test_bytes);

            let sa_naive = testing::naive_suffix_array(test_bytes);
            assert!(testing::compare_suffix_arrays(
                &mut suffix_array.iter(),
                &mut sa_naive.iter().copied()
            ));

            validate_suffix_array(&test_bytes, suffix_array.array());
        }
    }

    // Test building suffix array from random strings.
    fn test_random_strings(sa_builder: &dyn SuffixArrayBuilder) {
        // A primitive random function.
        fn next_random(num: &mut usize) -> usize {
            *num = (*num % 12345) * (*num % 2949) + 7;
            *num
        }

        const TEXT_LENGTH: usize = 1000;
        let mut test_bytes: Vec<u8> = vec![0; TEXT_LENGTH];
        let mut rand;
        for n in 0..1000 {
            rand = n;
            // Some primitive random number generator.
            for j in 0..TEXT_LENGTH {
                if j >= 1 && next_random(&mut rand) & 1 == 0 {
                    test_bytes[j] = test_bytes[j - 1];
                } else {
                    test_bytes[j] = ((next_random(&mut rand) % 16) + ('a' as usize)) as u8;
                }
            }

            if DEBUG_LEVEL >= 1 {
                println!(
                    "Test string: {}",
                    String::from_utf8_lossy(&test_bytes).to_string()
                );
            }

            let suffix_array = sa_builder.build(&test_bytes);
            let sa_naive = testing::naive_suffix_array(&test_bytes);
            assert!(testing::compare_suffix_arrays(
                &mut suffix_array.iter(),
                &mut sa_naive.iter().copied()
            ));

            validate_suffix_array(&test_bytes, suffix_array.array());
        }
    }

    #[test]
    fn all_tests() {
        use super::super::sa_is::SaIsBuilder;
        let builder = SaIsBuilder::new();
        test_short_strings(&builder);
        test_random_strings(&builder);
    }
}
