# Eludrs

A simple asynchronous wrapper for the Eludris API

## Installation

Just run the following command:

```sh
cargo add --git eludrs
```

## Example

While an API wrapper has many uses, here's an example of what most people will
end up using this for, making Bots:

```rust
use eludrs::HttpClient;
use futures::stream::StreamExt;

#[tokio::main]
async fn main() {
    let mut http = HttpClient::new().name("Uwuki".to_string());
    let gateway = http.create_gateway.await?;
    let mut events = gateway.get_events().await.unwrap();

    while let Some(msg) = events.next().await {
        if msg.content == "!ping" {
            http.send("Pong").await.unwrap();
        }
    }
}
```

## Docs

If you want documentation you can currently get that by going to your project
and running

```sh
cargo doc -p eludrs --open
```
