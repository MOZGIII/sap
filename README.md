# sap

Opinionated static file server for SPAs.

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
