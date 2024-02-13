use suffix_array::SaIsBuilder;
use suffix_array::NaiveBuilder;
use suffix_array::SuffixArrayBuilder;

// Whether to print verbose information for debugging.
#[allow(dead_code)]
const DEBUG_ENABLED: bool = false;

fn compute_suffix_array(sa_builder: Box<dyn SuffixArrayBuilder>, filename: &str) {
    println!("Reading file: {}", filename);
    let file_contents = std::fs::read_to_string(filename).unwrap();
    let text_bytes = file_contents.as_bytes();

    const REPETITIONS: usize = 1;
    for _ in 0.. REPETITIONS {
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
    }
    println!("Done");
}

fn main() -> std::io::Result<()> {
    let builder = Box::new(SaIsBuilder::new());
    //let builder = Box::new(NaiveBuilder::new());

    compute_suffix_array(builder, "/tmp/test.txt");
    Ok(())
}
