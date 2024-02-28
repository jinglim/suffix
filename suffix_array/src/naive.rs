use super::suffix_array::{SuffixArray, SuffixArrayBuilder, TextSize};

pub struct NaiveBuilder {}

impl NaiveBuilder {
    pub fn new() -> NaiveBuilder {
        NaiveBuilder {}
    }
}

struct NaiveSuffixArray {
    sa: Vec<TextSize>,
}

impl SuffixArray for NaiveSuffixArray {
    fn iter<'a>(&'a self) -> Box<dyn Iterator<Item = TextSize> + 'a> {
        Box::new(self.sa.iter().copied())
    }

    fn array(&self) -> &[TextSize] {
        &self.sa
    }
}

impl NaiveBuilder {
    fn build_suffix_array(text: &[u8]) -> Vec<TextSize> {
        let mut suffix_array: Vec<TextSize> = Vec::with_capacity(text.len());
        for i in 0..text.len() {
            suffix_array.push(i as TextSize);
        }
        suffix_array.sort_by(|&a, &b| text[a as usize..].cmp(&text[b as usize..]));
        suffix_array
    }
}

impl SuffixArrayBuilder for NaiveBuilder {
    // Naively computes a suffix array.
    fn build(&self, text: &[u8]) -> Box<dyn SuffixArray> {
        let sa: Vec<TextSize> = NaiveBuilder::build_suffix_array(text);
        Box::new(NaiveSuffixArray { sa })
    }
}
