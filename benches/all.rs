use std::fs;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use str_indices::chars;

fn all(c: &mut Criterion) {
    // Load benchmark strings.
    let test_strings: Vec<(&str, String)> = vec![
        (
            "en_10000",
            fs::read_to_string("benches/text/en_10000.txt").expect("Cannot find benchmark text."),
        ),
    ];

    //---------------------------------------------------------
    // Chars.

    // chars::count()
    {
        let mut group = c.benchmark_group("chars::count");
        for (text_name, text) in test_strings.iter() {
            group.bench_function(*text_name, |bench| {
                bench.iter(|| {
                    black_box(chars::count(text));
                })
            });
        }
    }
    {
        let mut group = c.benchmark_group("chars::count_inline");
        for (text_name, text) in test_strings.iter() {
            group.bench_function(*text_name, |bench| {
                bench.iter(|| {
                    black_box(chars::count_inline(text));
                })
            });
        }
    }
}

//-------------------------------------------------------------

criterion_group!(benches, all,);
criterion_main!(benches);
