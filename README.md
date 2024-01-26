<h1 align="center">Name Resolver</h1>
<br />
<p align="center">
<img width="250" src="https://i.imgur.com/nn7LMNV.png"/>
</p>
<p align="center">
<a href="https://twitter.com/bonfida">
<img src="https://img.shields.io/twitter/url?label=Bonfida&style=social&url=https%3A%2F%2Ftwitter.com%2Fbonfida">
</a>
</p>

<br />

<div align="center">
<img src="https://img.shields.io/badge/Cloudflare-F38020?style=for-the-badge&logo=Cloudflare&logoColor=white" />
<img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" />
<img src="https://img.shields.io/badge/WebAssembly-654FF0?style=for-the-badge&logo=WebAssembly&logoColor=white">
</div>

<br />
<a name="introduction"></a>
<h2 align="center">Introduction</h2>
<br />

![diagram](/assets/diagram.png)

This repository is an implementation of a Solana Name Service resolver made with Cloudflare workers.

It allows people to browse SNS websites directly from their favorite web browser (e.g [https://bonfida.sol-domain.org](https://bonfida.sol-domain.org)). The resolver supports URLs, [IPFS CIDs](https://www.ipfs.com/) and [Arweave hashes](https://www.arweave.org/).

When resolving a domain the worker will look into the following records and return the first one that exists:

- `url` record
- `IPFS` record
- `ARWV` record
- `SHDW` record
- `A` record
- `CNAME` record

The resolver will resolve both records v1 and records v2, but records v2 are given priority.

<br />
<h2 align="center">Deployment</h2>
<br />

This resolver is deployed on [https://sol-domain.org](https://sol-domain.org)

<br />
<h2 align="center">Get started</h2>
<br />

1. Install the wrangler CLI

```
yarn global add wrangler
```

2. Run the worker locally

```
wrangler dev
```

3. Deploy on Cloudflare

```
wrangler publish
```

<br />
<h2 align="center">Edit your records</h2>
<br />

To make your domain browsable, you must set your records:

- Go to your domain's page (e.g [https://naming.bonfida.org/domain/bonfida](https://naming.bonfida.org/domain/bonfida))
- Connect your wallet
- In order to resolve your domain one of the following record must be set:
  - `IPFS`: With the following format `ipfs://<CID>` (e.g `ipfs://QmZk9uh2mqmXJFKu2Hq7kFRh93pA8GDpSZ6ReNqubfRKKQ`)
  - `ARWV`: With the following format `arwv://<HASH>` (e.g `arwv://KuB5jmew87_M2flH9f6ZpB9jlDv8hZSHPrmGUY8KqEk`)
  - `url`: With the following format `url_to_your_website` (e.g `https://bonfida.org`)

![record](/assets/record.png)
