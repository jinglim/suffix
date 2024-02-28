use suffix_array::{SaIsBuilder, NaiveBuilder, SuffixArrayBuilder, validate_suffix_array};

// Whether to print verbose information for debugging.
#[allow(dead_code)]
const DEBUG_ENABLED: bool = false;

// Whether to validate the suffix array after it's been computed.
const VALIDATION_ENABLED: bool = false;

fn compute_suffix_array(sa_builders: &[Box<dyn SuffixArrayBuilder>], filename: &str) {
    println!("Reading file: {}", filename);
    let file_contents = std::fs::read_to_string(filename).unwrap();
    let text_bytes = file_contents.as_bytes();

    for sa_builder in sa_builders {
        println!("Building suffix array");
        let suffix_array = sa_builder.build(text_bytes);

        if DEBUG_ENABLED {
            for i in suffix_array.iter() {
                print!("{} ", i);
            }
            println!();

            for i in suffix_array.iter() {
                let end = std::cmp::min(text_bytes.len(), (i + 10) as usize);
                println!(
                    "{}: {}",
                    i,
                    String::from_utf8_lossy(&text_bytes[i as usize..end])
                );
            }
        }

        if VALIDATION_ENABLED {
            validate_suffix_array(&text_bytes, suffix_array.array());
        }
    }
    println!("Done");
}

fn main() -> std::io::Result<()> {
    let sa_is_builder = Box::new(SaIsBuilder::new());
    let naive_builder = Box::new(NaiveBuilder::new());

    //let builders : [Box<dyn SuffixArrayBuilder>; 2] = [sa_is_builder, naive_builder];
    let builders : [Box<dyn SuffixArrayBuilder>; 1] = [sa_is_builder];
    
    compute_suffix_array(&builders, "/tmp/test.txt");
    Ok(())
}
