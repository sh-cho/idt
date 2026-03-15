#![no_main]
use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;

use idt::IdKind;

#[derive(Arbitrary, Debug)]
struct Input {
    data: String,
    kind_index: u8,
}

fuzz_target!(|input: Input| {
    let kinds = IdKind::all();
    let kind = kinds[input.kind_index as usize % kinds.len()];
    let _ = idt::ids::parse_id(&input.data, Some(kind));
});
