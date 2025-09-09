mod roundtrip {
    use test_each_file::test_each_file;

    use crate::{decode, encode};

    test_each_file!( in "./brex/cases/"  => roundtrip);

    fn roundtrip(input: &str) {
        for line in input.lines() {
            println!("== {line:?}");
            let encoded = encode(line).unwrap().to_string();
            println!("-> {encoded:?}");
            let decoded = decode(&encoded).unwrap();

            assert_eq!(line, decoded)
        }
    }
}
