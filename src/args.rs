use argh::FromArgs;

#[derive(FromArgs)]
/// Download your podcast episoides.
///
/// Documentation at https://github.com/rohshall/podcaster-rs
pub struct Args {
    /// print the version
    #[argh(switch, short = 'v')]
    pub version: bool,

    #[argh(option, description = "podcast ID")]
    pub podcast_id: Option<String>,

    #[argh(option, description = "count of episodes")]
    pub count: Option<usize>,

    #[argh(positional, description = "show or download")]
    pub action: String,
}
