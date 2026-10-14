use std::net::TcpListener;

fn main() -> std::io::Result<()> {
    let config = jitter_config::load().unwrap_or_else(|error| {
        eprintln!("config error: {error}");
        std::process::exit(1);
    });
    let _tracing_guard = jitter_observability::init_tracing(&config);
    let addr = std::env::var("JITTER_SERVICE_ADDR").unwrap_or_else(|_| "127.0.0.1:8080".into());
    let listener = TcpListener::bind(&addr)?;
    println!("jitter-service listening on {addr}");
    jitter_service::serve(&listener, &config, usize::MAX)
}
