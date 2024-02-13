pub mod naive;
pub mod sa_is;
pub mod suffix_array;
mod testing;

pub use naive::NaiveBuilder;
pub use sa_is::SaIsBuilder;
pub use suffix_array::SuffixArrayBuilder;
