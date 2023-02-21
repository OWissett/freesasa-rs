use criterion::{
    black_box, criterion_group, criterion_main, Criterion,
};
use freesasa_rs::{result::*, set_fs_verbosity, structure::Structure};

fn load_structure() {
    let pdb_path = "./data/single_chain.pdb";
    for _ in 0..10 {
        let structure = Structure::from_path(pdb_path, None);
    }
}

pub fn structure_loading_benchmark(c: &mut Criterion) {
    c.bench_function("Structure Loading Benchmark", |b| {
        b.iter(|| load_structure())
    });
}

criterion_group!(benches, structure_loading_benchmark);
criterion_main!(benches);
