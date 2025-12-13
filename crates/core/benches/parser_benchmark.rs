use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use coding_guide_helper_core::{lexer::Lexer, parser::Parser};

// 小規模なCコードのパース
fn bench_parse_small(c: &mut Criterion) {
    let input = r#"
int main() {
    int x = 10;
    return x;
}
"#;

    c.bench_function("parse_small_function", |b| {
        b.iter(|| {
            let lexer = Lexer::new(black_box(input));
            let mut parser = Parser::new(lexer);
            parser.parse()
        })
    });
}

// 中規模なCコードのパース（制御構文含む）
fn bench_parse_medium(c: &mut Criterion) {
    let input = r#"
#define MAX 100

int fibonacci(int n) {
    if (n <= 1) {
        return n;
    }
    int a = 0;
    int b = 1;
    for (int i = 2; i; i) {
        int temp = a + b;
        a = b;
        b = temp;
    }
    return b;
}

int main() {
    int result = fibonacci(10);
    return result;
}
"#;

    c.bench_function("parse_medium_code", |b| {
        b.iter(|| {
            let lexer = Lexer::new(black_box(input));
            let mut parser = Parser::new(lexer);
            parser.parse()
        })
    });
}

// 構造体定義を含むコード
fn bench_parse_structs(c: &mut Criterion) {
    let input = r#"
struct Point {
    int x;
    int y;
};

struct Rectangle {
    struct Point top_left;
    struct Point bottom_right;
};

typedef struct {
    int width;
    int height;
} Size;

int calculate_area(struct Rectangle* rect) {
    int width = rect->bottom_right.x - rect->top_left.x;
    int height = rect->bottom_right.y - rect->top_left.y;
    return width * height;
}
"#;

    c.bench_function("parse_structs", |b| {
        b.iter(|| {
            let lexer = Lexer::new(black_box(input));
            let mut parser = Parser::new(lexer);
            parser.parse()
        })
    });
}

// プリプロセッサディレクティブが多いコード
fn bench_parse_preprocessor(c: &mut Criterion) {
    let input = r#"
#define DEBUG 1
#define VERSION 2

#ifdef DEBUG
    #define LOG(x) printf(x)
#else
    #define LOG(x)
#endif

#if VERSION >= 2
    int new_feature() {
        return 42;
    }
#endif

#ifndef HEADER_GUARD
#define HEADER_GUARD

int main() {
    #ifdef DEBUG
        LOG("Debug mode");
    #endif
    return 0;
}

#endif
"#;

    c.bench_function("parse_preprocessor", |b| {
        b.iter(|| {
            let lexer = Lexer::new(black_box(input));
            let mut parser = Parser::new(lexer);
            parser.parse()
        })
    });
}

// コードサイズによるパフォーマンス比較
fn bench_parse_by_size(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_by_size");
    
    for size in [10, 50, 100, 200].iter() {
        let mut input = String::new();
        for i in 0..*size {
            input.push_str(&format!("int var{} = {};\n", i, i));
        }
        
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                let lexer = Lexer::new(black_box(&input));
                let mut parser = Parser::new(lexer);
                parser.parse()
            })
        });
    }
    group.finish();
}

// レキサーのみのベンチマーク
fn bench_lexer_only(c: &mut Criterion) {
    let input = r#"
int main() {
    int x = 10;
    int y = 20;
    int z = x + y;
    return z;
}
"#;

    c.bench_function("lexer_tokenize", |b| {
        b.iter(|| {
            let mut lexer = Lexer::new(black_box(input));
            let mut tokens = Vec::new();
            while let Some(token) = lexer.next_token() {
                tokens.push(token);
            }
            tokens
        })
    });
}

criterion_group!(
    benches,
    bench_parse_small,
    bench_parse_medium,
    bench_parse_structs,
    bench_parse_preprocessor,
    bench_parse_by_size,
    bench_lexer_only
);
criterion_main!(benches);
