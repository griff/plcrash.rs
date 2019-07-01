extern crate addr2line;
extern crate plcrash;
extern crate uuid;

use std::fs::File;

use uuid::Uuid;

#[test]
fn it_adds_two() {
    let mut report_file = File::open("tests/MetaZ.plcrash").unwrap();
    let report = plcrash::read_report(&mut report_file).unwrap();
    let sym = plcrash::Symbolicate::new("tests/MetaZ-1.0-ec9e94c+dSYM.zip").unwrap();
    eprintln!("{}", plcrash::text_report(&report, Some(&sym)).unwrap());
    let uuid = Uuid::parse_str("5a537ce0-c887-3373-b4d9-2196436a4f14").unwrap();
    // 1   org.maven-group.MetaZ           0x0000000104489574 -[MZWriteQueue addQueueItems:] + 198 (MZWriteQueue.m:408)
    // 1   MetaZ                               0x0000000104489574 0x10447a000 + 62836

    for frame in sym.frames(&uuid, 62836).unwrap() {
        let frame = frame.unwrap();
        eprintln!("{}", frame);
    }
    eprintln!("Testing {:?}", sym.lookup(&uuid, 62836).unwrap());
    assert_eq!(sym.lookup(&uuid, 62836).unwrap(), Some(plcrash::Location(addr2line::Location {
        file: Some("/Users/bro/Documents/Maven-Group/MetaZ/App/src/MZWriteQueue.m".into()),
        line: Some(408),
        column: Some(5),
    })));

}
