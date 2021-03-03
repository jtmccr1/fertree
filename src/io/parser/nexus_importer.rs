use std::io::BufReader;

struct NexusParser<R>{
    reader:BufReader<R>
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    #[test]
    fn test(){
        let mut b = "This string will be read".as_bytes();
        char::from(4);
    }

}