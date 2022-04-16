const FILES_TO_CACHE = [
    "/",
    "/index.html",
    "/pairandomizer-pwa.js",
    "/pairandomizer-pwa_bg.wasm",
    "/res/icon.png",
];
const VERSION = 4;
const CACHE_NAME = `pairandomizer-v${VERSION}`;

self.addEventListener("install", e => {
    console.log(`[Service Worker] Version ${VERSION} Installing...`);
    e.waitUntil((async () => {
        const cache = await caches.open(CACHE_NAME);
        console.log('[Service Worker] Caching all: app shell and content');
        await cache.addAll(FILES_TO_CACHE);
    })());
});

self.addEventListener("message", e => {
    if (e.data === "force_update") {
        self.skipWaiting();
    }
});

const fetch_then_cache = async (request) => {
    const response = await fetch(request);
    const cache = await caches.open(CACHE_NAME);
    console.log(`[Service Worker] Caching new resource: ${request.url}`);
    cache.put(request, response.clone());
    return response;
};

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
        const r = await caches.match(e.request);
        console.log(`[Service Worker] Fetching resource: ${e.request.url}`);
        if (r) { return r; }
        return await fetch_then_cache(e.request);
    })());
});

self.addEventListener('activate', e => {
    e.waitUntil(caches.keys().then((keyList) => {
        return Promise.all(keyList.map((key) => {
            if (key === CACHE_NAME) { return; }
            return caches.delete(key);
        }))
    }));
});
