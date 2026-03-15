#![no_main]
use libfuzzer_sys::fuzz_target;

use idt::IdKind;

fuzz_target!(|kind_index: u8| {
    let kinds = IdKind::all();
    let kind = kinds[kind_index as usize % kinds.len()];
    if let Ok(generator) = idt::ids::create_generator(kind)
        && let Ok(id_str) = generator.generate()
        && let Ok(parsed) = idt::ids::parse_id(&id_str, Some(kind))
    {
        let canonical = parsed.canonical();
        // Re-parse canonical form should succeed
        let reparsed = idt::ids::parse_id(&canonical, Some(kind))
            .expect("canonical form should be re-parseable");
        assert_eq!(
            canonical,
            reparsed.canonical(),
            "canonical form should be stable"
        );
    }
});
