# sap

Opinionated static file server for SPAs.

## Usage

```dockerfile
FROM ghcr.io/mozgiii/sap:master

COPY . /app

ENV ROOT_DIR=/app
ENV ADDR=[::]:8080
```
