// ============================================================
// sw.js - Service worker: cache para jugar sin conexion
// ============================================================
// Un service worker es un script que el navegador ejecuta "al lado"
// de la pagina y que puede interceptar sus peticiones de red.
// Estrategia usada: "cache first" -> primero buscamos el archivo en
// el cache local; solo si no esta, vamos a internet.

// __CACHE_VERSION__ lo reemplaza GitHub Actions por el hash del commit.
// Al cambiar el nombre del cache, el navegador descarga la version nueva.
const CACHE_NAME = "gaterunner-__CACHE_VERSION__";

// Lista de archivos que forman el juego completo (el "app shell").
const ASSETS = [
    "./",
    "./index.html",
    "./mq_js_bundle.js",
    "./gaterunner.wasm",
    "./manifest.webmanifest",
    "./icon-192.png",
    "./icon-512.png",
    "./icon-512-maskable.png",
];

// "install": se dispara al instalar el service worker.
// Descargamos y guardamos todos los archivos en el cache.
self.addEventListener("install", (event) => {
    event.waitUntil(
        caches.open(CACHE_NAME)
            .then((cache) => cache.addAll(ASSETS))
            .then(() => self.skipWaiting()) // activa esta version sin esperar
    );
});

// "activate": se dispara cuando esta version toma el control.
// Borramos los caches de versiones anteriores para no acumular basura.
self.addEventListener("activate", (event) => {
    event.waitUntil(
        caches.keys()
            .then((keys) => Promise.all(
                keys.filter((key) => key !== CACHE_NAME)
                    .map((key) => caches.delete(key))
            ))
            .then(() => self.clients.claim())
    );
});

// "fetch": se dispara en cada peticion de la pagina.
// Respondemos desde el cache si es posible; si no, vamos a la red
// y guardamos la respuesta para la proxima vez.
self.addEventListener("fetch", (event) => {
    // Solo manejamos GET (no hay POST ni nada mas en este juego).
    if (event.request.method !== "GET") return;

    event.respondWith(
        caches.match(event.request).then((cached) => {
            if (cached) return cached;
            return fetch(event.request).then((response) => {
                // Solo cacheamos respuestas correctas de nuestro propio origen.
                if (response.ok && new URL(event.request.url).origin === self.location.origin) {
                    const copy = response.clone();
                    caches.open(CACHE_NAME).then((cache) => cache.put(event.request, copy));
                }
                return response;
            });
        })
    );
});
