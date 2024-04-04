use std::error::Error;
use std::fmt;
use std::fs;
use std::fs::File;
use std::io::BufWriter;
use std::sync::Mutex;
use tracing::dispatcher::DefaultGuard;
use tracing::{Event, Subscriber};
use tracing_subscriber::filter::{EnvFilter, LevelFilter};
use tracing_subscriber::fmt::format::Writer;
use tracing_subscriber::fmt::{FmtContext, FormatEvent, FormatFields};
use tracing_subscriber::registry::LookupSpan;

struct EventFormatter;

impl<S, N> FormatEvent<S, N> for EventFormatter
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &Event<'_>,
    ) -> fmt::Result {
        ctx.field_format().format_fields(writer.by_ref(), event)?;
        writeln!(writer)
    }
}

pub fn init() -> Result<DefaultGuard, Box<dyn Error>> {
    fs::create_dir_all("log")?;

    let file = File::create("log/cpu.log")?;
    let writer = BufWriter::new(file);

    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .with_env_var("LOG_LEVEL")
        .from_env()?;

    let subscriber = tracing_subscriber::fmt()
        .event_format(EventFormatter)
        .with_env_filter(env_filter)
        .with_writer(Mutex::new(writer))
        .finish();

    Ok(tracing::subscriber::set_default(subscriber))
}
