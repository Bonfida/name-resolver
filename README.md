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

This repository is an implementation of a Solana Name Service resolver made with Cloudflare workers. It allows people to browse websites with SNS record to be browsable from any web browser e.g `https://<domain_name>.your_deployment_url.com`. This code is deployed on `https://sol-domain.org` e.g [https://bonfida.sol-domain.org](https://bonfida.sol-domain.org)

When resolving a domain the worker will look into the following records and return the first one that exists:

- `url` record
- `IPFS` record
- `ARWV` record

The resolver supports sub-domains with the prioritization rule.

<br />
<a name="introduction"></a>
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
