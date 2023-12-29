// Files required for the app to work offline
const FILES_TO_CACHE = [
    "/",
    "/index.html",
    "/pairandomizer-pwa.js",
    "/pairandomizer-pwa_bg.wasm",
    "/res/icon.png",
];
// Cache name so we can install updates
const VERSION = 6;
const CACHE_NAME = `pairandomizer-v${VERSION}`;

// On install,
self.addEventListener("install", e => {
    console.log(`[Service Worker] Version ${VERSION} Installing...`);
    e.waitUntil((async () => {
        const cache = await caches.open(CACHE_NAME);
        // cache the FILES_TO_CACHE
        await cache.addAll(FILES_TO_CACHE);
    })());
});

// Optional: Expose a way to force an update
self.addEventListener("message", e => {
    if (e.data === "force_update") {
        self.skipWaiting();
    }
});

// Utility function to fetch something from the server, also storing it in cache.
const fetch_then_cache = async (request) => {
    const response = await fetch(request);
    const cache = await caches.open(CACHE_NAME);
    console.log(`[Service Worker] Caching new resource: ${request.url}`);
    cache.put(request, response.clone());
    return response;
};

// When the page attempts to fetch a resource,
self.addEventListener('fetch', e => {
    e.respondWith((async () => {
        if (e.request.url.includes('res/json')) {
            // fetch first, with fallback to cache
            try {
                return await fetch_then_cache(e.request);
            } catch (e) {
                console.log(
                    `[Service Worker] Failed to fetch from network, falling back to cache`
                );
            }
        }
        // we prefer to load of from the cache first
        const r = await caches.match(e.request);
        console.log(`[Service Worker] Fetching resource: ${e.request.url}`);
        if (r) { return r; }
        // only load it from the server if it isn't in the cache
        return await fetch_then_cache(e.request);
    })());
});

// Clean up after ourselves, deleting older versions
self.addEventListener('activate', e => {
    e.waitUntil(caches.keys().then((keyList) => {
        return Promise.all(keyList.map((key) => {
            if (key === CACHE_NAME) { return; }
            return caches.delete(key);
        }))
    }));
});
