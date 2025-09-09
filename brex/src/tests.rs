mod roundtrip {
    use test_each_file::test_each_file;

    use crate::{
        decode::{self, Brex},
        encode::encode,
    };

    test_each_file!( in "./brex/cases/"  => roundtrip);

    fn roundtrip(input: &str) {
        for line in input.lines() {
            println!("== {line:?}");
            let encoded = encode(line).unwrap().to_string();
            println!("-> {encoded:?}");
            let decoded = Brex::parse(&encoded).unwrap().1;
            let decoded = decoded.unroll();

            let line = line
                .strip_suffix(".bin")
                .expect(".bin to exist in input data");
            assert_eq!(line, decoded)
        }
    }
}
