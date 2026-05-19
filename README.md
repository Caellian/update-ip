# update-ip

A minimal dynamic DNS updater. Resolves your public IPv4/IPv6 addresses and
upserts DNS records with a provider.

## Configuration

| Variable | Description |
|---|---|
| `DNS_RECORD_NAME` | Record to update (e.g. `pc.example.com`) |

### Cloudflare provider (`provider-cloudflare`)

| Variable | Description |
|---|---|
| `CLOUDFLARE_API_TOKEN` | API token with **DNS:Edit** permission |
| `CLOUDFLARE_ZONE_ID` | Zone ID for the domain |

### OpenDNS resolver (`resolver-opendns`)

No configuration required. Queries OpenDNS resolvers via raw UDP DNS to
determine public IPv4/IPv6 addresses. This is the most stable option, completely free, not rate-limited, and cheaper to do than sending HTTP requests.

## Usage

```sh
CLOUDFLARE_API_TOKEN=... CLOUDFLARE_ZONE_ID=... DNS_RECORD_NAME=pc.example.com update-ip
```

> [!NOTE]
> Required environment variables depend on [features](#features) used during the build.

Service files for [systemd](dist/systemd/), [OpenRC](dist/openrc/), and
[cron](dist/crontab) (for runit and other systems) are available in
[`dist/`](dist/). Edit the environment variables and install the appropriate
one for your system.

## Features

| Feature | Default | Description |
|---|---|---|
| `resolver-opendns` | yes | Resolve public IP via OpenDNS (raw UDP DNS query) |
| `provider-cloudflare` | yes | Update records via Cloudflare API |

## Building

```sh
cargo +nightly build --release --target x86_64-unknown-linux-gnu
```

The release binary is ~64K with LTO, stripping, and `panic = "immediate-abort"` enabled.
TLS is provided by the system's OpenSSL.

## License

This project is licensed under the MIT license. A copy of the license is provided in the [LICENSE](./LICENSE) file in the root of the repository.
