# nightingale-client
A nightingale client ready to use with twilight and serenity

This client is a simple implementation that covers the API of [Nightingale]

This implementation only supports a single node per client, but more nodes can be
used in order to distribute the load between machines.

To use it, add the following line to your Cargo.toml:
```toml
nightingale-client = { git = "https://github.com/AlvaroMS25/nightingale-client" }
```

Now select one of `twilight` and `serenity` features, this enables the client to be
used with those two libraries.

Find docs here: [docs]

Now open a connection and you're ready to go!
A bunch of examples can be found on the [examples] folder

[Nightingale]: https://github.com/AlvaroMS25/nightingale
[examples]: https://github.com/AlvaroMS25/nightingale-client/tree/master/examples
[docs]: https://alvaroms25.github.io/nightingale-client/nightingale_client/index.html
