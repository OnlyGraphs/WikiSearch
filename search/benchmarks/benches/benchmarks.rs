use criterion::{black_box, criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};
use index::{
    index::{Index},
    index_structs::{Citation, Document, Infobox},
    PreIndex,
};
use parser::ast::{BinaryOp, Query, UnaryOp};

use retrieval::search::{execute_query, preprocess_query, score_query};

use std::fmt::{Debug, Display};

use rand::{rngs::StdRng, seq::SliceRandom, Rng, SeedableRng};

pub const VOCAB: &'static [&'static str] = &[
    "the",
    "hello",
    "world",
    "untidy",
    "strong",
    "multiply",
    "belong",
    "colossal",
    "glue",
    "lake",
    "wrathful",
    "undesirable",
    "slim",
    "mist",
    "defiant",
    "popcorn",
    "glow",
    "organic",
    "bee",
    "righteous",
    "stiff",
    "vegetable",
    "visitor",
    "enormous",
    "bedroom",
    "foolish",
    "dream",
    "rigid",
    "religion",
    "juggle",
    "exist",
    "relax",
    "room",
    "gaudy",
    "broad",
    "cough",
    "hour",
    "lick",
    "wipe",
    "quizzical",
    "aloof",
    "owe",
    "puny",
    "judge",
    "depend",
    "contain",
    "reminiscent",
    "childlike",
    "tomatoes",
    "spoil",
    "well-to-do",
    "splendid",
    "squeeze",
];

#[derive(Debug)]
pub struct IndexBenchParameters {
    id: String,
    articles_count: u32,
    words_count: u32,
    links_count: u32,
}

impl Default for IndexBenchParameters {
    fn default() -> Self {
        Self {
            id: format!("default"),
            articles_count: 10000,
            words_count: 600,
            links_count: 50,
        }
    }
}

pub fn get_random_string(w: u32, rng: &mut StdRng) -> String {
    get_random_strings(w, rng).join(" ")
}

pub fn get_random_strings(w: u32, rng: &mut StdRng) -> Vec<String> {
    (0..w)
        .map(|_c| VOCAB.choose(rng).expect("empty vocabulary").to_string())
        .collect::<Vec<String>>()
}

/// builds index with n documents with w words and each with the given number of links
pub fn get_random_documents(p: &IndexBenchParameters) -> Vec<Box<Document>> {
    let ref mut rng = StdRng::seed_from_u64(69420); // <- Here we set the seed

    let mut docs: Vec<Box<Document>> = Vec::default();
    let words_main = ((p.words_count as f32) * 0.80) as u32;
    let words_citations = ((p.words_count as f32) * 0.10) as u32;
    let words_infobox = ((p.words_count as f32) * 0.10) as u32;

    for i in 0..p.articles_count {
        docs.push(Box::new(Document {
            doc_id: i,
            title: i.to_string(),
            categories: "".to_string(),
            last_updated_date: "".to_string(),
            namespace: 2,
            main_text: get_random_string(words_main, rng),
            article_links: (0..p.links_count)
                .map(|_| rng.gen_range(0..p.articles_count).to_string())
                .collect::<Vec<String>>()
                .join("\t"),
            infoboxes: vec![Infobox {
                itype: "infobox".to_string(),
                text: get_random_string(words_infobox, rng),
            }],
            citations: vec![Citation {
                text: get_random_string(words_citations, rng),
            }],
        }))
    }

    Vec::default()
}

pub fn get_random_query(p: &IndexBenchParameters) -> Box<Query> {
    let ref mut rng = StdRng::seed_from_u64(69420); // <- Here we set the seed

    let fq = Box::new(Query::FreetextQuery {
        tokens: get_random_strings(5, rng),
    });

    let dist_q = Box::new(Query::DistanceQuery {
        dst: 3,
        lhs: get_random_string(1, rng),
        rhs: get_random_string(1, rng),
    });

    let and_q = Box::new(Query::BinaryQuery {
        lhs: fq,
        op: BinaryOp::And,
        rhs: dist_q,
    });

    let phrase_q = Box::new(Query::PhraseQuery {
        tks: get_random_strings(5, rng),
    });

    let not_q = Box::new(Query::UnaryQuery {
        op: UnaryOp::Not,
        sub: phrase_q,
    });

    let or_q = Box::new(Query::BinaryQuery {
        lhs: and_q,
        op: BinaryOp::Or,
        rhs: not_q,
    });

    let rel_q = Box::new(Query::RelationQuery {
        root: rng.gen_range(0..p.articles_count),
        hops: 5,
        sub: Some(or_q),
    });

    rel_q
}

pub fn build_index_with_docs(docs: Vec<Box<Document>>) -> Index {
    let mut pre_idx = PreIndex::default();

    docs.into_iter().for_each(|d| {
        pre_idx
            .add_document(d)
            .expect("Benchmarking failed, could not add document");
    });

    Index::from_pre_index(pre_idx)
}

pub fn execute_query_with_index(idx: &Index, mut q: Box<Query>) {
    preprocess_query(&mut *q).unwrap();
    let mut postings = execute_query(&q, &idx);
    score_query(&q, &idx, &mut postings);
}

impl IndexBenchParameters {
    fn from_article_count(c: u32) -> Self {
        let mut o = Self::default();
        o.articles_count = c;
        o.id = o.articles_count.to_string();
        o
    }

    // fn from_word_count(c: u32) -> Self {
    //     let mut o = Self::default();
    //     o.words_count = c;
    //     o.id = o.words_count.to_string();
    //     o
    // }

    // fn from_link_count(c: u32) -> Self {
    //     let mut o = Self::default();
    //     o.links_count = c;
    //     o.id = o.links_count.to_string();
    //     o
    // }
}

impl Display for IndexBenchParameters {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}

pub fn index_build_time(c: &mut Criterion) {
    let mut group = c.benchmark_group("index build time: article size");
    group.sample_size(20);

    for size in [200000].iter() {
        let ref param = IndexBenchParameters::from_article_count(*size);
        group.bench_with_input(BenchmarkId::from_parameter(param), param, |b, i| {
            b.iter_batched(
                || get_random_documents(i),
                |docs| {
                    build_index_with_docs(black_box(docs));
                },
                BatchSize::PerIteration,
            )
        });
    }

    group.finish();
}

pub fn query_execution_time(c: &mut Criterion) {
    let mut group = c.benchmark_group("query execution time: article size");
    group.sample_size(20);

    for size in [100, 1000, 10000].iter() {
        let ref param = IndexBenchParameters::from_article_count(*size);
        group.bench_with_input(BenchmarkId::from_parameter(param), param, |b, i| {
            b.iter_batched(
                || {
                    let idx = build_index_with_docs(get_random_documents(i));

                    let qs = get_random_query(i);

                    (idx, qs)
                },
                |(idx, qs)| execute_query_with_index(idx, qs),
                BatchSize::PerIteration,
            )
        });
    }

    group.finish();
}

criterion_group!(benches, index_build_time, query_execution_time);
criterion_main!(benches);
