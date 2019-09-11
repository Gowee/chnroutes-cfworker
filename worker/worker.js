addEventListener('fetch', event => {
  event.respondWith(handleRequest(event.request))
})

/**
 * Fetch and log a request
 * @param {Request} request
 */
async function handleRequest(request) {
  const { routes_from_rir_stats  } = wasm_bindgen;
  await wasm_bindgen(wasm)
  try {
    const rir_stats = await fetchRIRStats("apnic");
    const result = routes_from_rir_stats(rir_stats, "CN");
    return new Response(result, { status: 200 })
  }
  catch (error) {
    console.log(error);
    return new Response(error, { status: 500 });
  }
}

async function fetchRIRStats(registry) {
  // https://www.nro.net/about/rirs/statistics/
  const response = await fetch(`https://ftp.apnic.net/apnic/stats/${registry}/delegated-${registry}-latest`);
  if (!response.ok) {
    return new Error(`Failed to request upstream with HTTP ${response.status} (${response.statusText}).`);
  }
  return await response.text();
}