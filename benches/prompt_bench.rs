use criterion::{Criterion, criterion_group, criterion_main};
use git2::Repository;
use std::env;
use std::hint::black_box;
use tempfile::TempDir;

// Mock test setup helper
fn setup_test_repo() -> (TempDir, Repository) {
    let temp_dir = TempDir::new().unwrap();
    let repo = Repository::init(temp_dir.path()).unwrap();

    // Configure repo
    {
        let mut config = repo.config().unwrap();
        config.set_str("user.name", "Test User").unwrap();
        config.set_str("user.email", "test@example.com").unwrap();
    }

    // Create initial commit
    {
        let signature = repo.signature().unwrap();
        let tree_id = {
            let mut index = repo.index().unwrap();
            index.write_tree().unwrap()
        };
        let tree = repo.find_tree(tree_id).unwrap();
        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            "Initial commit",
            &tree,
            &[],
        )
        .unwrap();
    }

    (temp_dir, repo)
}

fn bench_get_env(c: &mut Criterion) {
    c.bench_function("get_env cached lookup", |b| {
        b.iter(|| {
            black_box(slick::get_env("SLICK_PROMPT_SYMBOL"));
            black_box(slick::get_env("SLICK_PROMPT_PATH_COLOR"));
            black_box(slick::get_env("SLICK_PROMPT_GIT_BRANCH_COLOR"));
        });
    });
}

fn bench_precmd_fast_path(c: &mut Criterion) {
    let (_temp_dir, repo) = setup_test_repo();
    let original_dir = env::current_dir().unwrap();

    c.bench_function("precmd fast path (branch, user, remote)", |b| {
        b.iter(|| {
            env::set_current_dir(repo.path().parent().unwrap()).unwrap();
            // This benchmarks the core git operations in the fast path
            let _ = black_box(repo.head());
            let _ = black_box(repo.config());
        });
    });

    env::set_current_dir(original_dir).unwrap();
}

fn bench_git_status(c: &mut Criterion) {
    let (_temp_dir, repo) = setup_test_repo();

    // Create some files for status checking
    std::fs::write(
        repo.path().parent().unwrap().join("test.txt"),
        "test content",
    )
    .unwrap();

    c.bench_function("git status check", |b| {
        b.iter(|| {
            let mut opts = git2::StatusOptions::new();
            opts.show(git2::StatusShow::IndexAndWorkdir);
            let statuses = repo.statuses(Some(&mut opts)).unwrap();
            black_box(statuses.len());
        });
    });
}

fn bench_env_cache_vs_direct(c: &mut Criterion) {
    let mut group = c.benchmark_group("env_cache_comparison");

    group.bench_function("cached get_env (10 calls)", |b| {
        b.iter(|| {
            for _ in 0..10 {
                black_box(slick::get_env("SLICK_PROMPT_SYMBOL"));
            }
        });
    });

    group.bench_function("direct env::var (10 calls)", |b| {
        b.iter(|| {
            for _ in 0..10 {
                black_box(env::var("SLICK_PROMPT_SYMBOL").unwrap_or_else(|_| "$".to_string()));
            }
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_get_env,
    bench_precmd_fast_path,
    bench_git_status,
    bench_env_cache_vs_direct
);
criterion_main!(benches);
