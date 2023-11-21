use common::{LogLevel, setup_logs_with_console_subscriber};
fn main() {
    setup_logs_with_console_subscriber(LogLevel::Debug).unwrap();
}