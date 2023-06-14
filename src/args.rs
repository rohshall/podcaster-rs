use argh::FromArgs;

#[derive(FromArgs)]
/// Download your podcast episoides.
///
/// Documentation at https://github.com/rohshall/podcaster-rs
pub struct Args {
    /// print the version
    #[argh(switch, short = 'v')]
    pub version: bool,

    /// list the available episodes
    #[argh(switch, short = 'l')]
    pub list: bool,

    /// download the available episodes
    #[argh(switch, short = 'd')]
    pub download: bool,
    
    #[argh(positional)]
    pub podcast_id: Option<String>,
}
