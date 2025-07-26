use std::sync::Arc;

use crate::command::config_get::ConfigGet;
use crate::command::echo::Echo;
use crate::command::get::Get;
use crate::command::info_replication::InfoReplication;
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
    InfoReplication(InfoReplication),
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
    async fn execute(&self, context: &CommandExecutorContext) -> Value;
}

pub fn parse(value: &Value) -> Result<CommandSet, anyhow::Error> {
    if let Ok(command) = Ping::parse_from(value) {
        return Ok(CommandSet::Ping(command));
    }
    if let Ok(command) = Echo::parse_from(value) {
        return Ok(CommandSet::Echo(command));
    }
    if let Ok(command) = Set::parse_from(value) {
        return Ok(CommandSet::Set(command));
    }
    if let Ok(command) = Get::parse_from(value) {
        return Ok(CommandSet::Get(command));
    }
    if let Ok(command) = Keys::parse_from(value) {
        return Ok(CommandSet::Keys(command));
    }
    if let Ok(command) = ConfigGet::parse_from(value) {
        return Ok(CommandSet::ConfigGet(command));
    }
    if let Ok(command) = InfoReplication::parse_from(value) {
        return Ok(CommandSet::InfoReplication(command));
    }
    Err(anyhow::anyhow!(
        "unable to parse value as any supported command"
    ))
}

pub async fn execute(command_set: CommandSet, context: &CommandExecutorContext) -> Value {
    match command_set {
        CommandSet::Ping(command) => command.execute(context).await,
        CommandSet::Echo(command) => command.execute(context).await,
        CommandSet::Set(command) => command.execute(context).await,
        CommandSet::Get(command) => command.execute(context).await,
        CommandSet::Keys(command) => command.execute(context).await,
        CommandSet::ConfigGet(command) => command.execute(context).await,
        CommandSet::InfoReplication(command) => command.execute(context).await,
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
