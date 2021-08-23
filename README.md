# chnroutes-cfworker
Generate <abbr title="routes of mainland China">chnroutes</abbr> by fetching [RIR stats](https://www.apnic.net/about-apnic/corporate-documents/documents/resource-guidelines/rir-statistics-exchange-format/) and managing to compressing routes by merging consecutive CIDRs, deployed on [Cloudflare Workers](https://workers.cloudflare.com/). Ported from [my Python script](https://gist.github.com/Gowee/3d06f1b96fbbeaba651604fd49b1458e).

<p align="center">
  <b>📢 The CIDR aggregator is now available as a standalone app: https://github.com/Gowee/cidr-aggregator 📢</b>
</p>
    
## API
**Base URL: `https://chnroutes-worker.bamboo.workers.dev` [🔗](https://chnroutes-worker.bamboo.workers.dev)**

### Generate Routes
- **Endpoint: `/generate`**
- **Parameters:**
    - `countries`
        - Comma-separated list of countries represented by 2 letters [Country Code](https://en.wikipedia.org/wiki/ISO_3166-1_alpha-2).
        - A leading `!` indicates excluding all following countries. For example, `!CN` will cover all countries/regions except mainland China in registries specified below. A single `!` indicates all countries. And so on...
    - `registries`
        - Comma-separated list of [registries](https://en.wikipedia.org/wiki/Regional_Internet_registry): AFRINIC, APNIC, ARIN, LACNIC, RIPE. Or `All` for all the five. 
        - Excluding is not supported.
- **Examples:**
    - `https://chnroutes-worker.bamboo.workers.dev/generate?countries=CN&registries=APNIC` [🔗](https://chnroutes-worker.bamboo.workers.dev/generate?countries=CN&registries=APNIC)
        - Typical chnroutes, useful for spliting traffic when bypassing the [GFW](https://en.wikipedia.org/wiki/Great_Firewall) with VPN/Proxies 
        - Becasuse almost all of mainland China IPs are delegated to APNIC currently (Sep 2019), so it might be sufficient to only cover APNIC.
    - `https://chnroutes-worker.bamboo.workers.dev/generate6?countries=US&registries=All` [🔗](https://chnroutes-worker.bamboo.workers.dev/generate?countries=US&registries=All)
        - All U.S. IPv6s delegated to any registries.
        - *The `/generate6` endpoint is experimental and subject to change.*

## Deployment
1. Make sure Rust and Cargo is installed.
2. Download the project with `git clone https://github.com/Gowee/chnroutes-cfworker`.
3. Install the CLI tool [wrangler](https://github.com/cloudflare/wrangler) for deploying to CloudFlare workers.
4. Configure and publish the worker following [the official quickstart](https://developers.cloudflare.com/workers/quickstart/#configure).

## Credits
This repo takes [rustwasm-worker-template](https://github.com/cloudflare/rustwasm-worker-template), which is a project by [Ashley Williams](https://github.com/ashleygwilliams), as the base skeleton.

The name chnroutes comes from https://github.com/fivesheep/chnroutes.git.
