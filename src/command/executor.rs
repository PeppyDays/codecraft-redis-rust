use std::sync::Arc;

use crate::command::config_get::ConfigGet;
use crate::command::echo::Echo;
use crate::command::get::Get;
use crate::command::keys::Keys;
use crate::command::ping::Ping;
use crate::command::set::Set;
use crate::config::Config;
use crate::repository::Repository;
use crate::resp::Value;

pub trait Command: Sized {
    fn parse_from(value: &Value) -> Result<Self, anyhow::Error>;
}

pub enum CommandSet {
    Ping(Ping),
    Echo(Echo),
    Set(Set),
    Get(Get),
    Keys(Keys),
    ConfigGet(ConfigGet),
}

#[derive(Clone)]
pub struct CommandExecutorContext {
    pub repository: Arc<dyn Repository>,
    pub config: Arc<Config>,
}

impl CommandExecutorContext {
    pub fn new(repository: Arc<dyn Repository>, config: Arc<Config>) -> Self {
        Self { repository, config }
    }
}

#[async_trait::async_trait]
pub trait CommandExecutor {
    async fn execute(&self, context: CommandExecutorContext) -> Value;
}

pub fn parse(value: &Value) -> Result<CommandSet, anyhow::Error> {
    if let Ok(ping) = Ping::parse_from(value) {
        return Ok(CommandSet::Ping(ping));
    }
    if let Ok(echo) = Echo::parse_from(value) {
        return Ok(CommandSet::Echo(echo));
    }
    if let Ok(set) = Set::parse_from(value) {
        return Ok(CommandSet::Set(set));
    }
    if let Ok(get) = Get::parse_from(value) {
        return Ok(CommandSet::Get(get));
    }
    if let Ok(keys) = Keys::parse_from(value) {
        return Ok(CommandSet::Keys(keys));
    }
    if let Ok(config_get) = ConfigGet::parse_from(value) {
        return Ok(CommandSet::ConfigGet(config_get));
    }
    Err(anyhow::anyhow!(
        "unable to parse value as any supported command"
    ))
}

pub async fn execute(command: CommandSet, context: CommandExecutorContext) -> Value {
    match command {
        CommandSet::Ping(ping) => ping.execute(context).await,
        CommandSet::Echo(echo) => echo.execute(context).await,
        CommandSet::Set(set) => set.execute(context).await,
        CommandSet::Get(get) => get.execute(context).await,
        CommandSet::Keys(keys) => keys.execute(context).await,
        CommandSet::ConfigGet(config_get) => config_get.execute(context).await,
    }
}

#[cfg(test)]
pub mod fixture {
    use std::sync::Arc;

    use crate::command::executor::CommandExecutorContext;
    use crate::config::Config;
    use crate::repository::Repository;
    use crate::repository::fixture::DummyRepository;

    #[rstest::fixture]
    pub fn command_executor_context(
        #[default(DummyRepository)] repository: impl Repository,
        #[default(Config::default())] config: Config,
    ) -> CommandExecutorContext {
        CommandExecutorContext::new(Arc::new(repository), Arc::new(config))
    }
}
