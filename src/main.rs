mod podcaster;
mod args;
use {
    crate::{
        args::*,
        podcaster::*
    },
};


fn main() {
    let args: Args = argh::from_env();
    if args.version {
        println!("podcaster {}", env!("CARGO_PKG_VERSION"));
        return;
    }
    let mut podcaster = Podcaster::new();

    match args.action {
        Action::DOWNLOAD => {
            match podcaster.download(args.podcast_id, args.count) {
                Ok(()) => println!("All is well."),
                Err(e) => eprintln!("Error while downloading podcasts {:?}", e)
            }
        },
        Action::LIST => {
            podcaster.list(args.podcast_id, args.count);
        },
        Action::CATCHUP => {
            podcaster.catchup(args.podcast_id, args.count);
        },
        Action::PLAY => {
            podcaster.play(args.podcast_id, args.count);
        }
    }
}
