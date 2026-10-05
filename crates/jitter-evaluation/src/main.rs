fn main() {
    println!(
        "{}",
        serde_json::to_string_pretty(&jitter_evaluation::run()).unwrap()
    )
}
