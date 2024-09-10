# podcaster
A podcast downloader written in Rust.

## Features
1. Reads config file in the home directory `~/.podcasts.json`  to get the config about which podcasts to download, and where to store the episodes. A sample config file `sample-podcasts.json` is included.
2. Downloads configurable number of episodes.
3. Remembers the episodes that were downloaded, so that even if the episode is deleted later, it does not re-download it.

### Coming soon

1. Download selected episodes.

## Usage
```
<program-name> [download/remote/show/play] [-p/--podcast-id <podcastId>] [-c/--count <count of episodes>]
```
For example:
```
podcaster download -p BegToDiffer -c 5
```

Downloads latest 5 episodes of the "BegToDiffer" podcast.

```
podcaster download
```
Downloads latest episode of all podcasts.

`remote` shows the podcast episodes available for download.
`download` downloads the podcast episodes.
`local` shows the podcast episodes downloaded locally for playing.
`play` plays the episodes downloaded. It uses the `player` from the config file. And it plays the latest episodes at the speed set by `speed` from the config. Currently, this works only if the player is `mpv`.

