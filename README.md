# Fast Dictionary API
I wrote this API for another application I was working on that needed definitions for English words quickly. I did not find anything suitable
so I built this. The data is prefetched from [Wiktionary](https://wiktionary.org) and stored in a SQLite database. The definition data is not shared since I'm not sure if that would be OK with Wiktionary.

I was debating whether to write this in Go or Rust and decided to give Rust a try. It's my first time using Rust; I would appreciate any feedback.

## Hosted API
https://dictionary.khoister.io

## Building the Docker image
```bash
docker build . -t fast-dictionary-api
```

## Running the Docker image
```bash
docker run --rm -d -p 8080:8080 fast-dictionary-api
```
Browse to http://localhost:8080
