# chnroutes-cfworker
Generate <abbr title="routes of mainland China">chnroutes</abbr> by fetching [RIR stats](https://www.apnic.net/about-apnic/corporate-documents/documents/resource-guidelines/rir-statistics-exchange-format/) from APNIC and managing to compressing routes by merging consecutive CIDRs, deployed on [Cloudflare Workers](https://workers.cloudflare.com/). 

## Credits
This repo takes [rustwasm-worker-template](https://github.com/cloudflare/rustwasm-worker-template), which is a project by [Ashley Williams](https://github.com/ashleygwilliams), as the base skeleton.

The name *chnroutes* comes from [https://github.com/fivesheep/chnroutes.git](https://github.com/fivesheep/chnroutes.git).