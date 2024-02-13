#[cfg(test)]
pub mod testing {
    use super::super::suffix_array::TextSize;

    // Naively computes a suffix array.
    pub fn naive_suffix_array(text: &[u8]) -> Vec<TextSize> {
        let mut suffix_array: Vec<TextSize> = Vec::with_capacity(text.len());
        for i in 0..text.len() {
            suffix_array.push(i as TextSize);
        }
        suffix_array.sort_by(|&a, &b| text[a as usize..].cmp(&text[b as usize..]));
        suffix_array
    }

    // Compare two suffix arrays and output differences if any.
    pub fn compare_suffix_arrays(
        a: &mut dyn Iterator<Item = TextSize>,
        b: &mut dyn Iterator<Item = TextSize>,
    ) -> bool {
        let mut pass = true;
        for i in 0.. {
            let a_next = a.next();
            let b_next = b.next();
            if a_next.is_none() {
                return pass && b_next.is_none();
            }
            if b_next.is_none() {
                return false;
            }
            let a_value = a_next.unwrap();
            let b_value = b_next.unwrap();
            if a_value != b_value {
                println!("Mismatched at {}: {} vs {} ", i, a_value, b_value);
                pass = false;
            }
        }
        println!("Failed.");
        println!();
        false
    }
}
