pub fn init() {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "uma_pilot=info,warn".into());
    let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();
}
