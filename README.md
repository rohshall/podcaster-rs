# podcaster
A podcast downloader written in Rust.

## Features
1. Reads config file in the home directory `~/.podcasts.json`  to get the config about which podcasts to download, and where to store the episodes. A sample config file `sample-podcasts.json` is included.
2. Downloads configurable number of episodes.

### Coming soon

1. Support show to list the episodes available for download.
2. Handle errors well.
3. Show listing - local and remote episodes
4. Download episodes using chapter numbers

## Usage
```
<program-name> [show|download] [--podcast-id <podcastId>] [--count 3]
```
For example:
```
podcaster download --podcast-id TheBulwark --count 5
```

It will download the latest 5 episodes of the "TheBulwark" podcast.


