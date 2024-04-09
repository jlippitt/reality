use std::collections::HashMap;
use std::error::Error;
use std::fmt::Debug;
use std::fs::File;
use std::io::{self, BufWriter, Stderr, Write};
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use tracing::field::{Field, Visit};
use tracing::metadata::ParseLevelError;
use tracing::span::{Attributes, Id, Record};
use tracing::subscriber::{DefaultGuard, Interest};
use tracing::{Event, Level, Metadata, Subscriber};

const ENV_VAR_NAME: &str = "LOG_LEVEL";
const DEFAULT_KEY: &str = "main";
const DEFAULT_LEVEL: Level = Level::DEBUG;
const LOG_BUFFER_SIZE: usize = 262144;

struct FieldVisitor<'a, T: Write> {
    writer: &'a mut T,
}

impl<'a, T: Write> FieldVisitor<'a, T> {
    pub fn new(writer: &'a mut T) -> Self {
        Self { writer }
    }
}

impl<'a, T: Write> Visit for FieldVisitor<'a, T> {
    fn record_debug(&mut self, field: &Field, value: &dyn Debug) {
        if field.name() == "message" {
            writeln!(self.writer, "{:?}", value).unwrap();
        }
    }
}

struct LogRouter {
    default_level: Level,
    writer_map: HashMap<&'static str, usize>,
    level_map: HashMap<String, Level>,
    writers: Vec<BufWriter<File>>,
    levels: Vec<Level>,
    stack: Vec<usize>,
    stderr: Stderr,
}

impl LogRouter {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        std::fs::create_dir_all("./log")?;

        let default_writer = create_writer(DEFAULT_KEY)?;

        let mut writer_map = HashMap::new();
        writer_map.insert(DEFAULT_KEY, 0);

        let mut level_map = HashMap::new();

        let default_level = if let Ok(level_string) = std::env::var(ENV_VAR_NAME) {
            parse_log_level(&mut level_map, &level_string)?
        } else {
            DEFAULT_LEVEL
        };

        Ok(Self {
            default_level,
            writer_map,
            level_map,
            writers: vec![default_writer],
            levels: vec![default_level],
            stack: vec![0],
            stderr: std::io::stderr(),
        })
    }

    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        let span_id = self.stack[self.stack.len() - 1];
        metadata.level() <= &self.levels[span_id]
    }

    pub fn new_span(&mut self, span: &Attributes<'_>) -> Id {
        let name = span.metadata().name();

        let span_id = self.writer_map.get(name).copied().unwrap_or_else(|| {
            let span_id = self.writers.len();
            self.writer_map.insert(name, span_id);

            let writer = create_writer(name).unwrap();
            self.writers.push(writer);

            let level = self
                .level_map
                .get(name)
                .copied()
                .unwrap_or(self.default_level);

            self.levels.push(level);

            span_id
        });

        // All span IDs are offset by 1, as 0 is reserved for the top-level writer
        Id::from_u64(span_id as u64 + 1)
    }

    pub fn event(&mut self, event: &Event<'_>) {
        let span_id = self.stack[self.stack.len() - 1];
        let writer = &mut self.writers[span_id];
        event.record(&mut FieldVisitor::new(writer));

        // If level if WARN or higher, additionally write to stderr
        if *event.metadata().level() <= Level::WARN {
            event.record(&mut FieldVisitor::new(&mut self.stderr));
        }
    }

    pub fn enter(&mut self, span: &Id) {
        self.stack.push(span.into_u64() as usize - 1);
    }

    pub fn exit(&mut self, span: &Id) {
        debug_assert!(self.stack[self.stack.len() - 1] == span.into_u64() as usize - 1);
        self.stack.pop();
    }
}

struct LogSubscriber {
    router: Arc<Mutex<LogRouter>>,
}

impl LogSubscriber {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let router = Arc::new(Mutex::new(LogRouter::new()?));

        Ok(Self { router })
    }
}

impl Subscriber for LogSubscriber {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        self.router.try_lock().unwrap().enabled(metadata)
    }

    fn new_span(&self, span: &Attributes<'_>) -> Id {
        self.router.try_lock().unwrap().new_span(span)
    }

    fn record(&self, _span: &Id, _values: &Record<'_>) {
        // Nothing
    }

    fn record_follows_from(&self, _span: &Id, _follows: &Id) {
        // Nothing
    }

    fn event(&self, event: &Event<'_>) {
        self.router.try_lock().unwrap().event(event)
    }

    fn enter(&self, span: &Id) {
        self.router.try_lock().unwrap().enter(span)
    }

    fn exit(&self, span: &Id) {
        self.router.try_lock().unwrap().exit(span)
    }

    fn register_callsite(&self, _metadata: &'static Metadata<'static>) -> Interest {
        Interest::sometimes()
    }
}

pub fn init() -> Result<DefaultGuard, Box<dyn Error>> {
    let subscriber = LogSubscriber::new()?;
    let guard = tracing::subscriber::set_default(subscriber);
    Ok(guard)
}

fn create_writer(name: &str) -> io::Result<BufWriter<File>> {
    let file = File::create(format!("./log/{}.log", name))?;
    let writer = BufWriter::with_capacity(LOG_BUFFER_SIZE, file);
    Ok(writer)
}

fn parse_log_level(
    level_map: &mut HashMap<String, Level>,
    level_string: &str,
) -> Result<Level, ParseLevelError> {
    let mut default_level = DEFAULT_LEVEL;

    for element in level_string.split(',') {
        if let Some((key, value)) = element.split_once('=') {
            level_map.insert(key.to_string(), Level::from_str(value)?);
        } else {
            default_level = Level::from_str(element)?;
        }
    }

    Ok(default_level)
}
