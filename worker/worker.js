addEventListener('fetch', event => {
  event.respondWith(handleRequest(event.request))
})

class ClientError extends Error {}

const { routes_from_rir_stats } = wasm_bindgen;

/**
 * Fetch and log a request
 * @param {Request} request
 */
async function handleRequest(request) {
  await wasm_bindgen(wasm)
  const url = new URL(request.url);
  switch (url.pathname) {
    case '/':
      return Response.redirect("https://github.com/Gowee/chnroutes-cfworker#api", 302);
      break;
    case '/generate':
      return await handleGenerate(request);
      break;
    default:
      return new Response(`Resource Not Found at Endpoint ${url.pathname}`, { status: 404 });
  }
}

async function handleGenerate(request) {
  try {
    const params = (new URL(request.url)).searchParams;
    const countries = (params.get("countries") || "!").toUpperCase();
    const registries = ((!params.get("registries") || params.get("registries").toUpperCase() == "ALL") ? "AFRINIC,APNIC,ARIN,LACNIC,RIPE" : params.get("registries").toUpperCase()).split(",");
    if (countries.match(/^\!?([A-Z]{2})?(,[A-Z]{2})*$/) === null) {
      throw new ClientError("Unable to parse `countries`");
    }
    const rir_stats = [];
    for (const registry of registries) {
      const data = await fetch_rir_stats(registry);
      rir_stats.push(data);
    }
    return new Response(routes_from_rir_stats(rir_stats.join("\n"), countries), { contentType: "text/plain" });
    //return new Response("Boom");
  }
  catch (e) {
    if (e instanceof ClientError) {
      return new Response(`Client Error: ${e}`, { status: 400 });
    }
    else {
      return new Response(`Server Error: ${e}`, { status: 500 });
    }
  }
}

async function fetch_rir_stats(registry) {
  const rir_stats_url = get_rir_stats_url(registry);
  if (!rir_stats_url) {
    throw new ClientError(`Unknown registry ${registry}`);
  }
  const cache = caches.default;
  let response = await cache.match(rir_stats_url);
  if (!response) {
    response = await fetch(rir_stats_url);
    if (!response.ok) {
      throw new Error(`Failed to request upstream with HTTP ${response.status} (${response.statusText}).`);
    }
    await cache.put(rir_stats_url, response.clone()); // FIX: waitUntil?; TODO: will CF handle expiration properly?
  }
  else {
    throw new Error("Cache hit!");
  }
  return await response.text();
}

function get_rir_stats_url(registry) {
  // https://www.nro.net/about/rirs/statistics/
  switch (registry) {
    case "AFRINIC":
      return "http://ftp.afrinic.net/pub/stats/afrinic/delegated-afrinic-latest";
    case "APNIC":
      return "http://ftp.apnic.net/stats/apnic/delegated-apnic-latest";
    case "ARIN":
      return "http://ftp.arin.net/pub/stats/arin/delegated-arin-extended-latest";
    case "LACNIC":
      return "http://ftp.lacnic.net/pub/stats/lacnic/delegated-lacnic-latest";
    case "RIPE":
      return "https://ftp.ripe.net/pub/stats/ripencc/delegated-ripencc-latest";
    default:
      return null;
  }
}
