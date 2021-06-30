pub mod clades;
pub mod annotate;
pub mod extract;
pub mod resolve;
pub mod split;
pub mod stats;
pub mod transmission_lineage;
pub mod branchlengths;

pub mod command_io {
    use csv::Reader;
    use std::error::Error;
    use std::fs::File;
    use std::path;
    use std::collections::HashSet;
    use std::io::{BufReader, BufRead};

    //HashMap<String,HashMap<String,AnnotationValue>>
    pub fn parse_tsv(trait_file: &path::Path) -> Result<Reader<File>, Box<dyn Error>> {
        let file = File::open(trait_file)?;
        let mut rdr = csv::ReaderBuilder::new()
            .delimiter(b'\t')
            .flexible(true)
            .comment(Some(b'#'))
            .from_reader(file);

        // We nest this call in its own scope because of lifetimes.
        debug!("read with headers:{:?}", rdr.headers().unwrap());

        Ok(rdr)
    }

    //HashMap<String,HashMap<String,AnnotationValue>>
    pub fn parse_taxa(taxa_file: Option<path::PathBuf>) -> Result<HashSet<String>, Box<dyn Error>> {
        Ok(match taxa_file {
            None => { HashSet::new() }
            Some(f) => {
                let mut taxa = HashSet::new();
                let file = File::open(f)?;
                let reader = BufReader::new(file);

                for line in reader.lines() {
                    taxa.insert(line?.trim().to_string());
                }
                debug!("{} taxa to ignore", taxa.len());
                taxa
            }
        }
        )
    }
}

