pub fn add(left: usize, right: usize) -> usize {
    left + right
}

pub struct Options {
    parse: ParseOptions,
}

pub struct ParseOptions {
    use_64_bit_numbers: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
