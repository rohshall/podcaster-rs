# podcaster
A podcast downloader written in Rust.

## Features
1. Reads config file in the home directory `~/.podcasts.toml`  to get the config about which podcasts to download, and where to store the episodes. A sample config file `sample-podcasts.toml` is included.
2. Downloads configurable number of episodes.
3. Remembers the episodes that were downloaded, so that even if the episode is deleted later, it does not re-download it.

### Coming soon

1. Download selected episodes.

## Usage
```
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
`play` plays the episodes downloaded. It uses the `player` from the config file. And it plays the latest episodes at the speed set by `playback_speed` from the config. Currently, this works only if the player is `mpv`.

