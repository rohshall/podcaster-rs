use std::fmt;
use core::str::FromStr;
use argh::FromArgs;

pub enum Action {
    LIST,
    CATCHUP,
    DOWNLOAD,
    PLAY
}

#[derive(Debug)]
pub struct ParseActionError {
    input: String
}

impl fmt::Display for ParseActionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_fmt(format_args!("Invalid value {}; allowed values are download, remote, local and play", self.input))
    }
}

impl FromStr for Action {
    type Err = ParseActionError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "download" => Ok(Action::DOWNLOAD),
            "list" => Ok(Action::LIST),
            "catchup" => Ok(Action::CATCHUP),
            "play" => Ok(Action::PLAY),
            _ => Err(ParseActionError{input: String::from(value)})
        }
    }
}


#[derive(FromArgs)]
/// Download your podcast episodes.
///
/// Documentation at https://github.com/rohshall/podcaster-rs
pub struct Args {
    /// print the version
    #[argh(switch, short = 'v')]
    pub version: bool,

    #[argh(option, short = 'p', description = "podcast ID")]
    pub podcast_id: Option<String>,

    #[argh(option, short = 'c', description = "count of episodes")]
    pub count: Option<usize>,

    #[argh(positional, default = "Action::DOWNLOAD", description = "action - download, list, catchup, play")]
    pub action: Action,
}
