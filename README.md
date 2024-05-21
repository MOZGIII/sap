# sap

Opinionated static file server for SPAs.

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
FROM ghcr.io/mozgiii/sap:master-onbuild
```

or using the regular image and copying the files manually:

```dockerfile
FROM ghcr.io/mozgiii/sap:master

COPY . /app

ENV ROOT_DIR=/app
```
