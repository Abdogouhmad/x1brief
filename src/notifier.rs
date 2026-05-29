use notify_rust::{Notification, Timeout};

pub struct X1BriefNotifier {
    app_name: String,
    timeout_ms: u32,
}

impl X1BriefNotifier {
    pub fn new() -> Self {
        Self {
            app_name: "X1Brief".to_string(),
            timeout_ms: 5000,
        }
    }

    pub fn notify(&self, title: &str, message: &str) {
        Notification::new()
            .summary(title)
            .body(message)
            .appname(&self.app_name)
            .timeout(Timeout::Milliseconds(self.timeout_ms))
            .show()
            .unwrap();
    }
}
