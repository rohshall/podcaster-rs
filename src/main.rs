mod podcaster;
mod settings;
mod podcast;
mod episode;

const HELP: &str = "\
podcaster

USAGE:
  podcaster [OPTIONS] [ACTION]

FLAGS:
  -h, --help            Prints help information

OPTIONS:
  -p String             podcast ID
  -c NUMBER             count of episodes

ARGS:
  ACTION                Supported actions are: download, list, catchup, and play. The default action is list.
  ";


#[derive(Debug)]
pub struct Args {
    podcast_id: Option<String>,
    count: Option<usize>,
    action: Option<String>,
}

fn parse_args() -> Result<Args, pico_args::Error> {
    let mut pargs = pico_args::Arguments::from_env();

    // Help has a higher priority and should be handled separately.
    if pargs.contains(["-h", "--help"]) {
        print!("{}", HELP);
        std::process::exit(0);
    }

    let args: Args = Args {
        podcast_id: pargs.opt_value_from_str("-p")?,
        count: pargs.opt_value_from_str("-c")?,
        action: pargs.opt_free_from_str()?
    };

    Ok(args)
}

fn main() {
    let args = parse_args().expect("Failed to parse args");
    let podcaster = podcaster::Podcaster::new();

    match args.action.unwrap_or("list".to_string()).as_str() {
        "download" => {
            podcaster.download(args.podcast_id, args.count);
        },
        "catchup" => {
            podcaster.catchup(args.podcast_id, args.count);
        },
        "play" => {
            podcaster.play(args.podcast_id, args.count);
        },
        "list" | _ => {
            podcaster.list(args.podcast_id, args.count);
        },
    }
}
