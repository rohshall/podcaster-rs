# podcaster
A simple podcast downloader written in Rust.

## Features
1. Reads config file in the config directory `<config-dir>/podcasts.toml` to get the config about which podcasts to download, and where to store the episodes. A sample config file `sample-podcasts.toml` is included. `<config-dir>` is specific to the OS. On Linux, it is `~/.config/podcaster-rs`; on Mac, it is `~/Library/Application Support/podcaster-rs`.
2. Downloads configurable number of episodes.
3. Remembers the episodes that were downloaded, so that even if the episode is deleted later, it does not re-download it.

### Coming soon

1. Download selected episodes.

## Usage
```
podcaster:  a simple podcast downloader.

USAGE:
  podcaster [OPTIONS] [ACTION]

FLAGS:
  -h, --help            Prints help information

OPTIONS:
  -p String             podcast ID
  -c NUMBER             count of episodes

ARGS:
  ACTION                Supported actions are: download, list and catchup. The default action is list.
```
For example:
```
podcaster download -p BegToDiffer -c 5
```

Downloads latest 5 episodes of the "BegToDiffer" podcast.
The default action is `download`. And the default count of episodes for download is `1`.

```
podcaster download
```
Downloads latest episode of all podcasts.

`list` shows the podcast episodes. It marks the ones which are not yet downloaded with `*`.
`download` downloads the podcast episodes.
`catchup` marks the podcast episodes as downloaded. This is useful when you are not interested in downloading the episodes after listing them.

