use std::fs::OpenOptions;
use std::sync::Arc;

use serde_json::{json, Value};
use tracing::field::{Field, Visit};
use tracing::{Event, Subscriber};
use tracing_subscriber::filter::Directive;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::layer::{Context, Layer, SubscriberExt};
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Registry};

use crate::config::AppConfig;
use crate::fmt;

pub fn init(config: &AppConfig) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let timer = tracing_subscriber::fmt::time::OffsetTime::local_rfc_3339()
        .expect("local time offset must be available");

    let mut env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&config.log_level));

    let directives = [
        "serenity=warn",
        "tokio_tungstenite=warn",
        "h2=warn",
        "dotacord=trace",
        "dotacord::data::player_matches_db=info",
    ];

    for directive in directives {
        if let Ok(parsed) = directive.parse::<Directive>() {
            env_filter = env_filter.add_directive(parsed);
        }
    }

    let stdout_layer = default_layer()
        .with_writer(std::io::stdout)
        .with_timer(timer.clone());

    let text_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&config.log_path)?;
    let text_file_layer = default_layer()
        .pretty()
        .with_writer(Arc::new(text_file))
        .with_timer(timer.clone())
        .with_ansi(false);

    let json_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&config.log_json_path)?;
    let json_file_layer = default_layer()
        .json()
        .with_writer(Arc::new(json_file))
        .with_timer(timer)
        .with_ansi(false);

    if let Some(seq_endpoint) = &config.seq_endpoint {
        Registry::default()
            .with(env_filter)
            .with(stdout_layer)
            .with(text_file_layer)
            .with(json_file_layer)
            .with(SeqLayer {
                endpoint: seq_endpoint.clone(),
            })
            .try_init()?;
    } else {
        Registry::default()
            .with(env_filter)
            .with(stdout_layer)
            .with(text_file_layer)
            .with(json_file_layer)
            .try_init()?;
    }

    Ok(())
}

fn default_layer<S>() -> tracing_subscriber::fmt::Layer<S>
where
    S: Subscriber,
{
    tracing_subscriber::fmt::layer()
        .with_level(true)
        // .with_thread_ids(true)
        // .with_thread_names(true)
        .with_line_number(true)
        // .with_target(true)
        // .with_file(true)
        .with_span_events(FmtSpan::CLOSE)
}

struct SeqLayer {
    endpoint: String,
}

struct SeqVisitor {
    fields: serde_json::Map<String, Value>,
}

impl SeqVisitor {
    fn new() -> Self {
        Self {
            fields: serde_json::Map::new(),
        }
    }
}

impl Visit for SeqVisitor {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        self.fields
            .insert(field.name().to_string(), json!(fmt!("{:?}", value)));
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        self.fields.insert(field.name().to_string(), json!(value));
    }

    fn record_i64(&mut self, field: &Field, value: i64) {
        self.fields.insert(field.name().to_string(), json!(value));
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        self.fields.insert(field.name().to_string(), json!(value));
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        self.fields.insert(field.name().to_string(), json!(value));
    }

    fn record_f64(&mut self, field: &Field, value: f64) {
        self.fields.insert(field.name().to_string(), json!(value));
    }
}

impl<S: Subscriber> Layer<S> for SeqLayer {
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let metadata = event.metadata();
        let level = match *metadata.level() {
            tracing::Level::TRACE => "Verbose",
            tracing::Level::DEBUG => "Debug",
            tracing::Level::INFO => "Information",
            tracing::Level::WARN => "Warning",
            tracing::Level::ERROR => "Error",
        };

        let mut visitor = SeqVisitor::new();
        event.record(&mut visitor);

        let message_template = visitor
            .fields
            .remove("message")
            .and_then(|v| v.as_str().map(String::from))
            .unwrap_or_else(|| fmt!("{}", metadata.name()));

        let mut payload = json!({
            "@t": chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Micros, true),
            "@mt": message_template,
            "@l": level,
            "SourceContext": metadata.target(),
        });

        if let Some(obj) = payload.as_object_mut() {
            if let Some(file) = metadata.file() {
                obj.insert("SourceFile".to_string(), json!(file));
            }
            if let Some(line) = metadata.line() {
                obj.insert("SourceLine".to_string(), json!(line));
            }
            for (key, value) in visitor.fields {
                obj.insert(key, value);
            }
        }

        let endpoint = self.endpoint.clone();
        let json_string = payload.to_string();

        std::thread::spawn(move || {
            match ureq::post(&endpoint)
                .set("Content-Type", "application/vnd.serilog.clef")
                .send_string(&json_string)
            {
                Ok(_) => {}
                Err(ureq::Error::Status(code, response)) => {
                    eprintln!(
                        "Seq rejected log event (HTTP {}): {}",
                        code,
                        response.into_string().unwrap_or_default()
                    );
                }
                Err(e) => {
                    eprintln!("Failed to send log to Seq: {}", e);
                }
            }
        });
    }
}
