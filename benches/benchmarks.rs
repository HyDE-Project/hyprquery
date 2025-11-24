//! Benchmarks for hyprquery operations.
//!
//! Run with: `cargo bench`
//! Results are saved to `target/criterion/`

use std::{fs, hint::black_box, io::Write};

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use hyprquery::{cli::Args, query::parse_query_inputs};

/// Sample HyDE-style configuration for benchmarking.
const BENCH_CONFIG: &str = r#"
$GTK_THEME = Gruvbox-Retro
$ICON_THEME = Gruvbox-Plus-Dark
$CURSOR_THEME = Bibata-Modern-Ice
$CURSOR_SIZE = 24
$terminal = kitty
$editor = nvim
$fileManager = dolphin
$menu = rofi

general {
    border_size = 2
    no_border_on_floating = false
    gaps_in = 5
    gaps_out = 10
    col.active_border = rgba(33ccffee) rgba(00ff99ee) 45deg
    col.inactive_border = rgba(595959aa)
    layout = dwindle
}

decoration {
    rounding = 8
    active_opacity = 1.0
    inactive_opacity = 0.9
    blur {
        enabled = true
        size = 3
        passes = 1
    }
    shadow {
        enabled = true
        range = 4
        render_power = 3
    }
}

animations {
    enabled = true
    bezier = myBezier, 0.05, 0.9, 0.1, 1.05
    animation = windows, 1, 7, myBezier
    animation = windowsOut, 1, 7, default, popin 80%
    animation = border, 1, 10, default
    animation = fade, 1, 7, default
    animation = workspaces, 1, 6, default
}

input {
    kb_layout = us,ru
    kb_options = grp:alt_shift_toggle
    follow_mouse = 1
    sensitivity = 0
    touchpad {
        natural_scroll = true
    }
}

misc {
    force_default_wallpaper = 0
    disable_hyprland_logo = true
}
"#;

/// Create a temporary config file for benchmarking.
fn create_bench_config() -> String {
    let temp_dir = std::env::temp_dir().join("hyprquery_bench");
    let _ = fs::create_dir_all(&temp_dir);
    let path = temp_dir.join("bench.conf");
    let mut file = fs::File::create(&path).expect("Failed to create bench config");
    write!(file, "{}", BENCH_CONFIG).expect("Failed to write bench config");
    path.to_string_lossy().to_string()
}

/// Create test Args for benchmarking.
fn make_bench_args(config_file: &str, queries: Vec<&str>) -> Args {
    Args {
        help:          false,
        config_file:   Some(config_file.to_string()),
        queries:       queries.into_iter().map(String::from).collect(),
        schema:        None,
        fetch_schema:  false,
        allow_missing: false,
        get_defaults:  false,
        strict:        false,
        export:        None,
        source:        false,
        debug:         false,
        delimiter:     "\n".to_string()
    }
}

/// Benchmark query parsing operations.
fn bench_query_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("query_parsing");

    group.bench_function("simple_key", |b| {
        b.iter(|| parse_query_inputs(&[black_box("general:border_size".to_string())]))
    });

    group.bench_function("with_type_filter", |b| {
        b.iter(|| parse_query_inputs(&[black_box("general:border_size[INT]".to_string())]))
    });

    group.bench_function("with_type_and_regex", |b| {
        b.iter(|| {
            parse_query_inputs(&[black_box("decoration:rounding[INT][^[0-9]+$]".to_string())])
        })
    });

    group.bench_function("dynamic_variable", |b| {
        b.iter(|| parse_query_inputs(&[black_box("$GTK_THEME".to_string())]))
    });

    // Benchmark with varying number of queries
    for count in [1, 5, 10] {
        let queries: Vec<String> = (0..count).map(|i| format!("general:key_{}", i)).collect();
        group.bench_with_input(
            BenchmarkId::new("multiple_queries", count),
            &queries,
            |b, q| b.iter(|| parse_query_inputs(black_box(q)))
        );
    }

    group.finish();
}

/// Benchmark file I/O operations.
fn bench_file_io(c: &mut Criterion) {
    let config_path = create_bench_config();
    let mut group = c.benchmark_group("file_io");

    group.bench_function("read_config", |b| {
        b.iter(|| {
            let content = fs::read_to_string(black_box(&config_path)).unwrap();
            black_box(content)
        })
    });

    group.finish();
}

/// Benchmark full query execution.
fn bench_full_execution(c: &mut Criterion) {
    let config_path = create_bench_config();
    let mut group = c.benchmark_group("full_execution");

    group.bench_function("single_static_key", |b| {
        let args = make_bench_args(&config_path, vec!["general:border_size"]);
        b.iter(|| {
            let _ = hyprquery::app::run_with_args(black_box(args.clone()));
        })
    });

    group.bench_function("single_with_type", |b| {
        let args = make_bench_args(&config_path, vec!["general:border_size[INT]"]);
        b.iter(|| {
            let _ = hyprquery::app::run_with_args(black_box(args.clone()));
        })
    });

    group.bench_function("multiple_queries", |b| {
        let args = make_bench_args(
            &config_path,
            vec![
                "general:border_size",
                "general:gaps_in",
                "decoration:rounding",
            ]
        );
        b.iter(|| {
            let _ = hyprquery::app::run_with_args(black_box(args.clone()));
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_query_parsing,
    bench_file_io,
    bench_full_execution
);
criterion_main!(benches);
