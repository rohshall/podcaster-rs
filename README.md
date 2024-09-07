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
<program-name> [show|download] [-p/--podcast-id <podcastId>] [-c/--count <count of episodes>]
```
For example:
```
podcaster download -p TheBulwark -c 5
```

Downloads latest 5 episodes of the "TheBulwark" podcast.

```
podcaster download
```
Downloads latest 3 episodes of all podcasts.
