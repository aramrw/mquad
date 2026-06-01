use std::sync::mpsc;
use tracing::field::{Field, Visit};
use tracing_core::{Event, Subscriber};
use tracing_subscriber::layer::Context;

pub struct ProgressLayer {
    pub sender: mpsc::Sender<String>,
}

impl<S: Subscriber> tracing_subscriber::Layer<S> for ProgressLayer {
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        if event.metadata().target().starts_with("yomichan_importer") {
            struct StringVisitor(String);
            impl Visit for StringVisitor {
                fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
                    if field.name() == "message" {
                        self.0 = format!("{:?}", value);
                    }
                }
            }
            let mut visitor = StringVisitor(String::new());
            event.record(&mut visitor);
            if !visitor.0.is_empty() {
                let _ = self.sender.send(visitor.0.trim_matches('"').to_string());
            }
        }
    }
}
