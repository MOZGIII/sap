# sap

Opinionated HTTP server for hosting static files of Single Page Apps from memory
with blazing fast speeds.

## Goals and non-goals

Goals:

- serve static files
- simple templating to allow injecting ENV-values-based configurations
- easy docker image composition
- sane configuration for select HTTP header values, like CSP and etc

Non-goals:

- SSR
- SPA code builds (you can still do this via your own docker stages)
- API proxying (already handled by the reverse proxy / edge)
- HTTPS (already handled by the reverse proxy / edge)

## Usage

Using the `onbuild` image:

```dockerfile
FROM ghcr.io/mozgiii/sap:latest-onbuild
```

or using the regular image and copying the files manually:

```dockerfile
FROM ghcr.io/mozgiii/sap:latest

COPY . /app

ENV ROOT_DIR=/app
```

## Configuration

### Server configuration

All the configuration parameters are set via env vars.

See <crates/sap/src/main.rs> to learn about the available settings.

### SPA configuration

There are two (opinionated) ways for configuring Single Page Applications
with `sap`:

1. substituting values from the env variables for the corresponding
   JSON object keys in the HTML `script` tag with the `type` attribute set
   to `application/spa-cfg` on the `/` route - this is called HTML templating,
   and
2. substituting values from the env variables for the corresponding
   JSON object keys in the JSON file on the `/config.json` route - this is
   called JSON templating.

#### General idea

The in both cases we take the JSON object. We iterate over the keys of
the object, we take each key, turn it into `CONSTANT_CASE` add prefix and
check if a env var with this name exists. If it doesn't - the value of the key
is left as it is in the original JSON, but if the env var **is** set - then
we replace the value in JSON with the value of the env var.

> [!WARNING]
>
> We require that JSON object used for configuration has only `string` values.
>
> In TypeScript that requirement would look like this:
>
> ```js
> type Config = { [key: string]: string };
> ```

#### HTML templating

Here is a simplified example of `index.html` file (used as the root (`/`) route):

```html
<html>
<body>
  <script type="application/spa-cfg">{"myKey": "default value"}</script>
</body>
</html>
```

With `sap` using default SPA configs prefix `APP_` and with
the `APP_MY_KEY` env var set to `override value`, this is what the returned
page body will look like:

```html
<html>
<body>
  <script type="application/spa-cfg">{"myKey": "override value"}</script>
</body>
</html>
```

##### Example with Docker (HTML templating)

If we assume that you have a `Dockerfile` like this:

```shell
$ cat Dockerfile
FROM ghcr.io/mozgiii/sap:latest
ENV ROOT_DIR /app
COPY build /app
```

and the `index.html` in the `build` directory like this:

```shell
$ cat build/index.html
<html>
<body>
  <script type="application/spa-cfg">{"myKey": "default value"}</script>
</body>
</html>
```

we can build the test image like this:

```shell
$ docker build . -t sap-test
...
```

At this point we have our app dockerized and ready to run.

```shell
$ docker run --rm -it -p 8080:8080 sap-test
...
2024-01-01T12:34:56.789123Z  Successfully applied HTML templating route=/ dir_entry_path="/app/index.html"
...
2024-01-01T12:34:56.789123Z  INFO sap: About to start the server addr=0.0.0.0:8080
...
```

In another terminal we can run the following command to try it:

```shell
$ curl http://localhost:8080
<html>
<body>
  <script type="application/spa-cfg">{"myKey": "default value"}</script>
</body>
</html>
```

The neat thing is that we can change the settings as we please, without
rebuilding the app from source or recreating docker image - just like
it was intended with containers!

Continue with stopping the `sap-test` container via `Ctrl+C`...

```shell
C^
... (stopping) ...
$
```

... and running with the env var to change the SPA configuration:

```shell
$ docker run --rm -it -p 8080:8080 -e APP_MY_KEY="override value" sap-test
...
2024-01-01T12:34:56.789123Z  Successfully applied HTML templating route=/ dir_entry_path="/app/index.html"
...
2024-01-01T12:34:56.789123Z  INFO sap: About to start the server addr=0.0.0.0:8080
...
```

Then in the other terminal:

```shell
$ curl http://localhost:8080
<html>
<body>
  <script type="application/spa-cfg">{"myKey": "override value"}</script>
</body>
</html>
```

Note how the configuration value `myKey` has changed.

##### Usage in the SPA code (HTML templating)

This can be utilized from the SPA with the code like this:

```js
function readConfig() {
  const el = document.querySelector("script[type=\"application/spa-cfg\"]");
  const configText = el.innerText;
  const config = JSON.parse(configText);
  return config;
}
```

This allows you to get the dynamic configuration without rebuilds. This way also
has a benefit of not involving any `async` or `Promise`s.

#### JSON templating

> [!IMPORTANT]
>
> JSON templating in an opt-in, and requires `CONFIG_JSON_TEMPLATING=true` env
> var to be set to work.

An example of `config.json` file (with the `/config.json` route):

```json
{"myKey":"default value"}
```

With `sap` with `CONFIG_JSON_TEMPLATING=true`, using default SPA configs prefix
`APP_`  and with the `APP_MY_KEY` env var set to `override value`, this is what
the returned `/config.json` response body will look like:

```json
{"myKey":"override value"}
```

##### Example with Docker (JSON templating)

If we assume that you have a `Dockerfile` like this:

```shell
$ cat Dockerfile
FROM ghcr.io/mozgiii/sap:latest
ENV ROOT_DIR /app
COPY build /app
```

and the `config.json` in the `build` directory like this:

```shell
$ cat build/config.json
{"myKey":"default value"}
```

we can build the test image like this:

```shell
$ docker build . -t sap-test
...
```

At this point we have our app dockerized and ready to run.

> Well, technically it is just the config, but you can add the rest of the files
> of your app there as well.

```shell
$ docker run --rm -it -p 8080:8080 sap-test
...
2024-01-01T12:34:56.789123Z  Successfully applied JSON templating route=/config.json dir_entry_path="/app/config.json"
...
2024-01-01T12:34:56.789123Z  INFO sap: About to start the server addr=0.0.0.0:8080
...
```

In another terminal we can run the following command to try it:

```shell
$ curl http://localhost:8080/config.json
{"myKey":"default value"}
```

The neat thing, just like with HTML templating, is that we can change
the settings as we please, without rebuilding the app from source or recreating
docker image - just like it was intended with containers!

Continue with stopping the `sap-test` container via `Ctrl+C`...

```shell
C^
... (stopping) ...
$
```

... and running with the env var to change the SPA configuration:

```shell
$ docker run --rm -it -p 8080:8080 -e APP_MY_KEY="override value" sap-test
...
2024-01-01T12:34:56.789123Z  Successfully applied JSON templating route=/config.json dir_entry_path="/app/config.json"
...
2024-01-01T12:34:56.789123Z  INFO sap: About to start the server addr=0.0.0.0:8080
...
```

Then in the other terminal:

```shell
$ curl http://localhost:8080/config.json
{"myKey":"override value"}
```

Again, just like with HTML templating, configuration value of the key `myKey`
has changed.

##### Usage in the SPA code (JSON templating)

This can be utilized from the SPA with the code like this:

```js
const readConfig = async () => fetch("/config.json").then(res => res.json());
```
