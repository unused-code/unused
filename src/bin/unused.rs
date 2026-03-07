#[cfg(all(feature = "mimalloc", unix, not(target_env = "musl")))]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn main() {
    cli::run();
}
