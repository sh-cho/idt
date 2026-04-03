use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};

use idt::core::detection::detect_id_type;
use idt::core::encoding::{encode_base32, encode_base58, encode_base64, encode_bits, encode_hex};
use idt::core::id::{IdGenerator, IdKind, ParsedId};
use idt::ids::{
    Cuid2Generator, CuidGenerator, KsuidGenerator, NanoIdGenerator, ObjectIdGenerator, ParsedKsuid,
    ParsedObjectId, ParsedSnowflake, ParsedTypeId, ParsedUlid, ParsedUuid, ParsedXid,
    SnowflakeGenerator, TsidGenerator, TypeIdGenerator, UlidGenerator, UuidGenerator, XidGenerator,
};

// ---------------------------------------------------------------------------
// Generation benchmarks
// ---------------------------------------------------------------------------

fn bench_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("generation");

    group.bench_function("uuid_v4", |b| {
        let generator = UuidGenerator::v4();
        b.iter(|| generator.generate().unwrap());
    });

    group.bench_function("uuid_v7", |b| {
        let generator = UuidGenerator::v7();
        b.iter(|| generator.generate().unwrap());
    });

    group.bench_function("ulid", |b| {
        let generator = UlidGenerator::new();
        b.iter(|| generator.generate().unwrap());
    });

    group.bench_function("nanoid", |b| {
        let generator = NanoIdGenerator::new();
        b.iter(|| generator.generate().unwrap());
    });

    group.bench_function("snowflake", |b| {
        let generator = SnowflakeGenerator::new();
        b.iter(|| generator.generate().unwrap());
    });

    group.bench_function("ksuid", |b| {
        let generator = KsuidGenerator::new();
        b.iter(|| generator.generate().unwrap());
    });

    group.bench_function("objectid", |b| {
        let generator = ObjectIdGenerator::new();
        b.iter(|| generator.generate().unwrap());
    });

    group.bench_function("xid", |b| {
        let generator = XidGenerator::new();
        b.iter(|| generator.generate().unwrap());
    });

    group.bench_function("tsid", |b| {
        let generator = TsidGenerator::new();
        b.iter(|| generator.generate().unwrap());
    });

    group.bench_function("cuid", |b| {
        let generator = CuidGenerator::new();
        b.iter(|| generator.generate().unwrap());
    });

    group.bench_function("cuid2", |b| {
        let generator = Cuid2Generator::new();
        b.iter(|| generator.generate().unwrap());
    });

    group.bench_function("typeid", |b| {
        let generator = TypeIdGenerator::new("");
        b.iter(|| generator.generate().unwrap());
    });

    group.finish();
}

fn bench_batch_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_generation");

    for count in [10, 100, 1000] {
        group.bench_function(format!("uuid_v4_{count}"), |b| {
            let generator = UuidGenerator::v4();
            b.iter(|| generator.generate_many(black_box(count)).unwrap());
        });

        group.bench_function(format!("ulid_{count}"), |b| {
            let generator = UlidGenerator::new();
            b.iter(|| generator.generate_many(black_box(count)).unwrap());
        });

        group.bench_function(format!("snowflake_{count}"), |b| {
            let generator = SnowflakeGenerator::new();
            b.iter(|| generator.generate_many(black_box(count)).unwrap());
        });
    }

    group.finish();
}

// ---------------------------------------------------------------------------
// Parsing benchmarks
// ---------------------------------------------------------------------------

fn bench_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("parsing");

    let uuid_str = UuidGenerator::v4().generate().unwrap();
    group.bench_function("uuid", |b| {
        b.iter(|| ParsedUuid::parse(black_box(&uuid_str)).unwrap());
    });

    let ulid_str = UlidGenerator::new().generate().unwrap();
    group.bench_function("ulid", |b| {
        b.iter(|| ParsedUlid::parse(black_box(&ulid_str)).unwrap());
    });

    let snow_str = SnowflakeGenerator::new().generate().unwrap();
    group.bench_function("snowflake", |b| {
        b.iter(|| ParsedSnowflake::parse(black_box(&snow_str)).unwrap());
    });

    let ksuid_str = KsuidGenerator::new().generate().unwrap();
    group.bench_function("ksuid", |b| {
        b.iter(|| ParsedKsuid::parse(black_box(&ksuid_str)).unwrap());
    });

    let oid_str = ObjectIdGenerator::new().generate().unwrap();
    group.bench_function("objectid", |b| {
        b.iter(|| ParsedObjectId::parse(black_box(&oid_str)).unwrap());
    });

    let xid_str = XidGenerator::new().generate().unwrap();
    group.bench_function("xid", |b| {
        b.iter(|| ParsedXid::parse(black_box(&xid_str)).unwrap());
    });

    let typeid_str = TypeIdGenerator::new("").generate().unwrap();
    group.bench_function("typeid", |b| {
        b.iter(|| ParsedTypeId::parse(black_box(&typeid_str)).unwrap());
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Detection benchmarks
// ---------------------------------------------------------------------------

fn bench_detection(c: &mut Criterion) {
    let mut group = c.benchmark_group("detection");

    let uuid_str = UuidGenerator::v4().generate().unwrap();
    group.bench_function("uuid", |b| {
        b.iter(|| detect_id_type(black_box(&uuid_str)).unwrap());
    });

    let ulid_str = UlidGenerator::new().generate().unwrap();
    group.bench_function("ulid", |b| {
        b.iter(|| detect_id_type(black_box(&ulid_str)).unwrap());
    });

    let snow_str = SnowflakeGenerator::new().generate().unwrap();
    group.bench_function("snowflake", |b| {
        b.iter(|| detect_id_type(black_box(&snow_str)).unwrap());
    });

    let ksuid_str = KsuidGenerator::new().generate().unwrap();
    group.bench_function("ksuid", |b| {
        b.iter(|| detect_id_type(black_box(&ksuid_str)).unwrap());
    });

    group.bench_function("invalid", |b| {
        b.iter(|| detect_id_type(black_box("!!not-an-id!!")));
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Encoding benchmarks
// ---------------------------------------------------------------------------

fn bench_encoding(c: &mut Criterion) {
    let mut group = c.benchmark_group("encoding");

    // Use 16 bytes (UUID-sized) as a representative payload
    let uuid = ParsedUuid::parse(&UuidGenerator::v4().generate().unwrap()).unwrap();
    let bytes = uuid.as_bytes();

    group.bench_function("hex", |b| {
        b.iter(|| encode_hex(black_box(&bytes)));
    });

    group.bench_function("base32", |b| {
        b.iter(|| encode_base32(black_box(&bytes)));
    });

    group.bench_function("base58", |b| {
        b.iter(|| encode_base58(black_box(&bytes)));
    });

    group.bench_function("base64", |b| {
        b.iter(|| encode_base64(black_box(&bytes)));
    });

    group.bench_function("bits", |b| {
        b.iter(|| encode_bits(black_box(&bytes)));
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Inspect benchmarks
// ---------------------------------------------------------------------------

fn bench_inspect(c: &mut Criterion) {
    let mut group = c.benchmark_group("inspect");

    let uuid = ParsedUuid::parse(&UuidGenerator::v4().generate().unwrap()).unwrap();
    group.bench_function("uuid", |b| {
        b.iter(|| uuid.inspect());
    });

    let ulid = ParsedUlid::parse(&UlidGenerator::new().generate().unwrap()).unwrap();
    group.bench_function("ulid", |b| {
        b.iter(|| ulid.inspect());
    });

    let snow = ParsedSnowflake::parse(&SnowflakeGenerator::new().generate().unwrap()).unwrap();
    group.bench_function("snowflake", |b| {
        b.iter(|| snow.inspect());
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Full pipeline: generate → parse → detect → inspect
// ---------------------------------------------------------------------------

fn bench_full_pipeline(c: &mut Criterion) {
    let mut group = c.benchmark_group("full_pipeline");

    group.bench_function("uuid_v4", |b| {
        let generator = UuidGenerator::v4();
        b.iter(|| {
            let id = generator.generate().unwrap();
            let parsed = idt::ids::parse_id(&id, Some(IdKind::UuidV4)).unwrap();
            let _ = parsed.inspect();
        });
    });

    group.bench_function("ulid", |b| {
        let generator = UlidGenerator::new();
        b.iter(|| {
            let id = generator.generate().unwrap();
            let parsed = idt::ids::parse_id(&id, Some(IdKind::Ulid)).unwrap();
            let _ = parsed.inspect();
        });
    });

    group.bench_function("uuid_v4_autodetect", |b| {
        let generator = UuidGenerator::v4();
        b.iter(|| {
            let id = generator.generate().unwrap();
            let parsed = idt::ids::parse_id(&id, None).unwrap();
            let _ = parsed.inspect();
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_generation,
    bench_batch_generation,
    bench_parsing,
    bench_detection,
    bench_encoding,
    bench_inspect,
    bench_full_pipeline,
);
criterion_main!(benches);
