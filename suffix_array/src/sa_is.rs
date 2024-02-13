// This is an implementation of the SA-IS suffix array construction algorithm.
// See: Nong, Ge; Zhang, Sen; Chan, Wai Hong (2009).
// Linear Suffix Array Construction by Almost Pure Induced-Sorting.

use super::suffix_array::{SuffixArray, SuffixArrayBuilder, TextSize};

// If enabled (> 0), perform more validations and output more debug info.
#[allow(dead_code)]
const DEBUG_LEVEL: usize = 0;

// Each suffix string is either a LType or SType.
#[derive(Copy, Clone, PartialEq)]
enum LSType {
    // Suffix at (pos) is LType if it is larger than suffix starting at (pos + 1).
    LType,

    // Suffix at (pos) is SType if it is smaller than suffix starting at (pos + 1).
    SType,
}
use LSType::*;

// Represents a null value as usize type.
const NULL: TextSize = TextSize::MAX;

// Represents a sequence of text characters.
trait Text: std::fmt::Display {
    // Length of the text.
    fn len(&self) -> TextSize;

    // Returns a character as u32.
    fn char_at(&self, index: TextSize) -> u32;

    // Returns a suffix string starting at given index. Used for debugging.
    fn suffix_at(&self, index: TextSize) -> String;

    // Compares two LMS substrings in this text.
    // The two substrings should have the same chars, and same LSType.
    fn lms_strings_equal(&self, mut a: TextSize, mut b: TextSize, ls_type: &[LSType]) -> bool {
        // LMS strings start with STypes.
        debug_assert!(ls_type[a as usize] == SType && ls_type[b as usize] == SType);

        // Loop until char mismatch or a LType is encountered.
        loop {
            if self.char_at(a) != self.char_at(b) {
                return false;
            }
            let a_type = ls_type[a as usize];
            if a_type == LType {
                if ls_type[b as usize] != LType {
                    return false;
                }
                break;
            }
            a += 1;
            b += 1;
        }

        // Loop until char mismatch or a SType is encountered.
        loop {
            a += 1;
            b += 1;
            let a_type = ls_type[a as usize];
            if a_type == SType {
                return ls_type[b as usize] == SType;
            }
            if self.char_at(a) != self.char_at(b) {
                return false;
            }
        }
    }
}

// An implementation of Text trait for u8 sequences.
struct ByteText<'a> {
    text: &'a [u8],
}

impl<'a> Text for ByteText<'a> {
    fn len(&self) -> TextSize {
        self.text.len() as TextSize
    }

    fn char_at(&self, index: TextSize) -> u32 {
        self.text[index as usize] as u32
    }

    fn suffix_at(&self, index: TextSize) -> String {
        String::from_utf8_lossy(&self.text[index as usize..]).to_string()
    }
}

impl<'a> std::fmt::Display for ByteText<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", String::from_utf8_lossy(self.text))?;
        Ok(())
    }
}

// An implementation of Text trait for u32 sequences.
struct U32Text<'a> {
    text: &'a [u32],
}

impl<'a> Text for U32Text<'a> {
    fn len(&self) -> TextSize {
        self.text.len() as TextSize
    }

    fn char_at(&self, index: TextSize) -> u32 {
        self.text[index as usize]
    }

    fn suffix_at(&self, index: TextSize) -> String {
        let mut suffix = String::new();
        for i in index as usize..self.text.len() {
            suffix += &format!("[{}] ", self.text[i]);
        }
        suffix
    }
}

impl<'a> std::fmt::Display for U32Text<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for i in self.text.iter() {
            write!(f, "[{}]", i)?;
        }
        Ok(())
    }
}

// Iterates through LMS positions in a given Text.
struct LmsIterator<'a> {
    // Array of LSTypes.
    ls_type: &'a [LSType],

    // Current position.
    pos: TextSize,
}

// Iterates through all the LMS positions, but not the last one at (len).
impl<'a> LmsIterator<'a> {
    // Creates a new iterator from a ls_type array.
    fn new(ls_type: &'a [LSType]) -> LmsIterator<'a> {
        LmsIterator { ls_type, pos: 0 }
    }
}

// Implements the iterator trait.
impl<'a> std::iter::Iterator for LmsIterator<'a> {
    type Item = TextSize;

    // Scan the ls_type array until the next (LType)(SType) sequence.
    fn next(&mut self) -> Option<Self::Item> {
        let ls_type = self.ls_type;

        // Scan until we get to LType. Note that ls_type[len - 1] == LType.
        let mut pos = self.pos as usize;
        while ls_type[pos] == SType {
            pos += 1;
        }
        pos += 1;

        // Scan until we get to SType. Note that ls_type[len] == SType.
        while ls_type[pos] == LType {
            pos += 1;
        }
        self.pos = pos as TextSize;

        // Ignore the very last LMS position at (len).
        if pos < self.ls_type.len() - 1 {
            Some(self.pos)
        } else {
            None
        }
    }
}

// A bucket represents all suffixes starting with the same first character.
// e.g. for "abac" text, The bucket for "a" will be {start: 0, end: 2}.
struct Bucket {
    // The starting pos in a sorted suffix array.
    start: TextSize,

    // The (last pos + 1) in a sorted suffix array.
    end: TextSize,
}

// Contains various data structures computed from the original text.
struct SuffixData {
    // Contains LSType of each char in the text.
    ls_type: Vec<LSType>,

    // The buckets in ascending order.
    // e.g. bucket[0] = bucket for char 0.
    buckets: Vec<Bucket>,

    // Indexes of buckets that are not empty.
    // This allows quicker iteration through the buckets.
    bucket_indexes: Vec<usize>,
}

// Builds a SuffixArray recursively.
struct RecursiveBuilder<'a> {
    text: &'a dyn Text,

    // This is 256 in the initial build, but may increase subsequently.
    // Assuming that subsequent alphabet will be less than 2^32.
    alphabet_size: u32,
}

// Result of reducing the original text to a new shorter text.
enum ReducedText {
    // Already sorted, don't need to recurse.
    Sorted {
        sorted_pos: Vec<TextSize>,
    },
    // Reduced to the given text and alphabet size.
    Reduced {
        text: Vec<u32>,
        alphabet_size: u32,

        // Translates the new text position to the original text position.
        pos_translation: Vec<TextSize>,
    },
}

// The SA-IS algorithm is recursive. This builder calls itself recursively to build
// subsequent smaller Text inputs.
impl<'a> RecursiveBuilder<'a> {
    // Creates a new instance.
    fn new(text: &'a dyn Text, alphabet_size: u32) -> RecursiveBuilder<'a> {
        RecursiveBuilder {
            text,
            alphabet_size,
        }
    }

    // Builds a suffix array and returns it.
    fn build(&mut self) -> Vec<TextSize> {
        if DEBUG_LEVEL >= 1 {
            println!(
                "  BUILD text:{} alphabet:{}",
                self.text.len(),
                self.alphabet_size
            );
        }

        // First, scan the text and build the data structures used for later computation.
        let suffix_data = self.build_suffix_data();

        // Run the induced sort, followed by a reduction step.
        let reduced_data = {
            let mut sa: Vec<TextSize> = vec![0; self.text.len() as usize];

            // Induced sort on the LMS substrings.
            let bucket_tails = self.induced_sort(
                &suffix_data,
                &mut sa,
                &mut LmsIterator::new(&suffix_data.ls_type),
            );

            // Reduce into smaller text, for recursion.
            self.reduce(&suffix_data, &sa, &bucket_tails)
        };

        // Sort all the suffixes starting from LMS positions.
        let sorted_lms: Vec<TextSize> = {
            match reduced_data {
                ReducedText::Sorted { sorted_pos } => sorted_pos,
                ReducedText::Reduced {
                    text,
                    alphabet_size,
                    pos_translation,
                } => {
                    // Expect the reduced text is at most half the original length.
                    debug_assert!(text.len() <= (self.text.len() / 2) as usize);

                    // Recurse on the new text.
                    let u32_text: U32Text = U32Text { text: &text };
                    let mut new_sa_builder = RecursiveBuilder::new(&u32_text, alphabet_size);
                    let new_sa = new_sa_builder.build();

                    // Convert the pos of the suffix array of the reduced text to the pos in
                    // the original text.
                    let mut sorted_lms = Vec::with_capacity(text.len());
                    for &pos in new_sa.iter() {
                        sorted_lms.push(pos_translation[pos as usize]);
                    }
                    sorted_lms
                }
            }
        };

        // Run another induced sort to sort the rest of the suffixes.
        let mut sa: Vec<TextSize> = vec![0; self.text.len() as usize];
        self.induced_sort(&suffix_data, &mut sa, &mut sorted_lms.iter().rev().copied());
        sa
    }

    // Print debugging info.
    #[allow(dead_code)]
    fn print(&self, title: &str, suffix_data: &SuffixData) {
        println!();
        println!("{}", title);
        println!("  text: {}", self.text);
        print!("  type: ");
        for ls in suffix_data.ls_type.iter() {
            print!(
                "{}",
                match ls {
                    LType => 'L',
                    SType => 'S',
                }
            );
        }
        println!();
        println!();
    }

    // Scan the text and compute the LSType of each character position, and char frequencies
    // for bucket sort.
    fn build_suffix_data(&self) -> SuffixData {
        let text = self.text;
        let len = text.len();

        // Classify all positions into LType or SType.
        let mut ls_type = vec![LType; text.len() as usize + 1];

        // Type at [len - 1] is always LType, and type at [len] is always SType.
        ls_type[len as usize] = SType;

        // Count number of each item in the text for bucketing.
        let mut char_count: Vec<TextSize> = vec![0; self.alphabet_size as usize];
        char_count[text.char_at(len - 1) as usize] += 1;

        if len >= 2 {
            // Scan from right to left, and set each SType position type.

            // Initial value for (len - 1).
            let mut next_ch = text.char_at(len - 1);
            let mut next_type = LType;

            // Iterate pos from (len - 2) to 0.
            for pos in (0..len - 1).rev() {
                let ch = text.char_at(pos);

                // Keep track of number of char occurences for bucketing.
                char_count[ch as usize] += 1;

                // The types are initialized to LTypes, so only update if the type is SType.
                if ch < next_ch {
                    ls_type[pos as usize] = SType;
                    next_type = SType;
                } else if next_type == SType {
                    // If the char at pos == char at (pos + 1), then type(pos) == type(pos + 1).
                    if ch == next_ch {
                        ls_type[pos as usize] = SType;
                    } else {
                        next_type = LType;
                    }
                }

                next_ch = ch;
            }
        }

        // Build the buckets, where each bucket corresponds to each char value in the text.
        let mut buckets: Vec<Bucket> = Vec::with_capacity(self.alphabet_size as usize);
        let mut bucket_indexes: Vec<usize> = Vec::with_capacity(self.alphabet_size as usize);
        let mut total_count: TextSize = 0;
        for (i, &count) in char_count.iter().enumerate() {
            buckets.push(Bucket {
                start: total_count,
                end: total_count + count,
            });
            total_count += count;

            // Bucket indexes contain only non-empty buckets.
            if count > 0 {
                bucket_indexes.push(i);
            }
        }
        assert!(total_count == self.text.len());

        SuffixData {
            ls_type,
            buckets,
            bucket_indexes,
        }
    }

    // Induced sorting.
    // stype_iter is an initial set of sorted SType positions to start the
    // induced sort.
    fn induced_sort(
        &self,
        suffix_data: &SuffixData,
        sa: &mut [TextSize],
        stype_iter: &'a mut dyn std::iter::Iterator<Item = TextSize>,
    ) -> Vec<TextSize> {
        // Assign a rank for prev_pos at the next available head pos of the
        // bucket. LTypes are always placed at the head part of the bucket.
        fn assign_ltype(
            text: &dyn Text,
            pos: TextSize,
            buckets: &[Bucket],
            bucket_heads: &mut [TextSize],
            sa: &mut [TextSize],
        ) {
            let ch = text.char_at(pos);
            let head_pos = bucket_heads[ch as usize];
            bucket_heads[ch as usize] += 1;
            sa[(buckets[ch as usize].start + head_pos) as usize] = pos;
        }

        // Assign a rank for prev_pos at the next tail pos of the bucket.
        // SType are always placed at the tail part of the bucket.
        fn assign_stype(
            text: &dyn Text,
            pos: TextSize,
            buckets: &[Bucket],
            bucket_tails: &mut [TextSize],
            sa: &mut [TextSize],
        ) {
            let ch = text.char_at(pos);
            bucket_tails[ch as usize] += 1;
            let tail_pos = bucket_tails[ch as usize];
            sa[(buckets[ch as usize].end - tail_pos) as usize] = pos;
        }

        let ls_type = &suffix_data.ls_type;
        let buckets = &suffix_data.buckets;

        // These provide information about the next available position at the
        // head/tail of each bucket.
        let mut bucket_heads: Vec<TextSize> = vec![0; self.alphabet_size as usize];
        let mut bucket_tails: Vec<TextSize> = vec![0; self.alphabet_size as usize];

        // Assign the last char, which is always a LType that is smallest in its bucket.
        assign_ltype(
            self.text,
            self.text.len() - 1,
            buckets,
            &mut bucket_heads,
            sa,
        );

        // Fill each LMS position at the tail of its bucket based on its first char.
        for pos in stype_iter {
            assign_stype(self.text, pos, buckets, &mut bucket_tails, sa);
        }

        // Traverse the buckets from left (lowest) to right (highest) and place LTypes at the head
        // of the buckets.
        for &b in suffix_data.bucket_indexes.iter() {
            let bucket = &buckets[b];

            // Assign the LTypes at the head of the bucket.
            let mut i = 0;
            while i < bucket_heads[b] {
                let pos = sa[(bucket.start + i) as usize];
                if pos > 0 {
                    let prev_pos = pos - 1;
                    if ls_type[prev_pos as usize] == LType {
                        assign_ltype(self.text, prev_pos, buckets, &mut bucket_heads, sa);
                    }
                }
                i += 1;
            }

            // Assign the LType before each LMS suffix in the bucket.
            for i in bucket.end - bucket_tails[b]..bucket.end {
                let pos = sa[i as usize];
                let prev_pos = pos - 1;
                assign_ltype(self.text, prev_pos, buckets, &mut bucket_heads, sa);
            }
        }

        // Reset the bucket tails for the next step.
        bucket_tails.fill(0);

        // Traverse the buckets from right to left, and fill STypes at the tail of the buckets.
        for &b in suffix_data.bucket_indexes.iter().rev() {
            let bucket = &buckets[b];

            // Traverse the S positions (at the tail of the bucket).
            let mut i = 0;
            while i < bucket_tails[b] {
                let pos = sa[(bucket.end - 1 - i) as usize];
                if pos > 0 {
                    let prev_pos = pos - 1;
                    if ls_type[prev_pos as usize] == SType {
                        assign_stype(self.text, prev_pos, buckets, &mut bucket_tails, sa);
                    }
                }
                i += 1;
            }

            // Traverse the L positions (at the head of the bucket).
            for i in (bucket.start..bucket.start + bucket_heads[b]).rev() {
                let pos = sa[i as usize];
                if pos > 0 {
                    let prev_pos = pos - 1;
                    if ls_type[prev_pos as usize] == SType {
                        assign_stype(self.text, prev_pos, buckets, &mut bucket_tails, sa);
                    }
                }
            }
        }

        // Return the number of tail positions in each bucket. This will be used in the reduce step.
        bucket_tails
    }

    // Run a reduce step and produce a new text that represents each LMS substrings in the original
    // text.
    fn reduce(
        &self,
        suffix_data: &SuffixData,
        sa: &[TextSize],
        bucket_tails: &[TextSize],
    ) -> ReducedText {
        let ls_type = &suffix_data.ls_type;
        let buckets = &suffix_data.buckets;

        // Number of LMS suffixes.
        let mut num_lms = 0;

        // Maps each LMS position to its new name.
        let mut pos_name_map: Vec<u32> = vec![0; self.text.len() as usize];

        // Stores LMS offsets in ascending order.
        let mut sorted_unique_pos: Vec<TextSize> =
            Vec::with_capacity((self.text.len() as usize) / 4); // Estimate.

        // Iterate through the bucekts.
        for &b in suffix_data.bucket_indexes.iter() {
            let bucket = &buckets[b];

            let mut last_lms_pos = NULL;

            // Iterate through the tails of each bucket, that's where the LMS suffixes are.
            for i in bucket.end - bucket_tails[b]..bucket.end {
                // Check if pos is a LMS.
                let pos = sa[i as usize];
                if pos > 0 && ls_type[(pos - 1) as usize] == LType {
                    debug_assert!(ls_type[pos as usize] == SType);

                    if last_lms_pos != NULL
                        && self.text.lms_strings_equal(last_lms_pos, pos, ls_type)
                    {
                        // The LMS substring is the same as previous.
                        pos_name_map[pos as usize] = (sorted_unique_pos.len() - 1) as u32;
                    } else {
                        // The LMS substring is a new unique substring.
                        pos_name_map[pos as usize] = sorted_unique_pos.len() as u32;
                        sorted_unique_pos.push(pos);
                        last_lms_pos = pos;
                    }
                    num_lms += 1;
                }
            }
        }

        if sorted_unique_pos.len() == num_lms {
            ReducedText::Sorted {
                sorted_pos: sorted_unique_pos,
            }
        } else {
            // Scan LMS positions and construct the new text by mapping each LMS substring to the
            // its new alphabet.
            let mut new_text: Vec<u32> = Vec::with_capacity(num_lms);
            let mut pos_translation: Vec<TextSize> = Vec::with_capacity(num_lms);

            for pos in LmsIterator::new(&suffix_data.ls_type) {
                new_text.push(pos_name_map[pos as usize]);
                pos_translation.push(pos);
            }

            ReducedText::Reduced {
                text: new_text,
                alphabet_size: sorted_unique_pos.len() as u32,
                pos_translation,
            }
        }
    }
}

// An implementation of SA-IS algorithm.
pub struct SaIsBuilder {}

impl SaIsBuilder {
    pub fn new() -> SaIsBuilder {
        SaIsBuilder {}
    }
}

struct SaIsSuffixArray {
    sa: Vec<TextSize>,
}

impl SuffixArray for SaIsSuffixArray {
    fn iter<'a>(&'a self) -> Box<dyn Iterator<Item = TextSize> + 'a> {
        Box::new(self.sa.iter().copied())
    }
}

impl SuffixArrayBuilder for SaIsBuilder {
    fn build(&self, text: &[u8]) -> Box<dyn SuffixArray> {
        // Check that the text size is supported.
        assert!(text.len() < (TextSize::MAX - 1) as usize);
        assert!(!text.is_empty());
        if DEBUG_LEVEL >= 1 {
            println!("Building Suffix Array");
        }
        let byte_text = ByteText { text };
        let mut sa_builder = RecursiveBuilder::new(&byte_text, 256);
        let sa = sa_builder.build();

        if DEBUG_LEVEL >= 2 {
            println!("Done");
            print_sorted_array("Sorted suffix array", &sa);
        }

        Box::new(SaIsSuffixArray { sa })
    }
}

fn print_sorted_array(title: &str, sa: &[TextSize]) {
    println!("{}:", title);
    for &i in sa.iter() {
        print!("{} ", i);
    }
    println!();
}