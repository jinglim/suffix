// Builds a SuffixArray.

// Assuming the size of text is less than 2^TextSize.
pub type TextSize = u32;

pub trait SuffixArrayBuilder {
    // Builds a suffix array from a given text.
    fn build(&self, text: &[u8]) -> Box<dyn SuffixArray>;
}

pub trait SuffixArray {
    fn iter<'a>(&'a self) -> Box<dyn Iterator<Item = TextSize> + 'a>;
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
            "abcbabcba",
            "cabbage abc food abc vegetables",
            "ktrthleluzsxleo",
            "fjhccfahejdacbffahbf",
            "wprnnuivivdygnarkzkjmvmpuxuzbsehrmunexkvkjczbbrawh",
            "zdzkixqfehvgmrqpevpqmnefcjsppmqxwbibbkylelyjcjulejunynoxypzbcjlw",
        ];
        for test_str in test_strings {
            let test_bytes = test_str.as_bytes();
            let suffix_array = sa_builder.build(test_bytes);

            let sa_naive = testing::naive_suffix_array(test_bytes);
            assert!(testing::compare_suffix_arrays(
                &mut suffix_array.iter(),
                &mut sa_naive.iter().copied()
            ));
        }
    }

    // Test building suffix array from random strings.
    fn test_random_strings(sa_builder: &dyn SuffixArrayBuilder) {
        // A primitive random function.
        fn next_random(num: &mut usize) -> usize {
            *num = (*num % 12345) * (*num % 2949) + 7;
            *num
        }

        const TEXT_LENGTH: usize = 500;
        let mut test_bytes: Vec<u8> = vec![0; TEXT_LENGTH];
        let mut rand = 1;
        for _ in 0..1000 {
            // Some primitive random number generator.
            for j in 0..TEXT_LENGTH {
                if j >= 1 && next_random(&mut rand) & 1 == 0 {
                    test_bytes[j] = test_bytes[j - 1];
                } else {
                    test_bytes[j] = ((next_random(&mut rand) % 16) + ('a' as usize)) as u8;
                }
            }
            println!("Test string: {}", String::from_utf8_lossy(&test_bytes).to_string());
    
            let suffix_array = sa_builder.build(&test_bytes);
            let sa_naive = testing::naive_suffix_array(&test_bytes);
            assert!(testing::compare_suffix_arrays(
                &mut suffix_array.iter(),
                &mut sa_naive.iter().copied()
            ));
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
