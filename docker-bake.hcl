group "default" {
  targets = ["main", "main-onbuild"]
}

target "main" {
  inherits = ["docker-metadata-action-main"]
  dockerfile = "Dockerfile"
  target = "main"
}

target "main-onbuild" {
  inherits = ["docker-metadata-action-main-onbuild"]
  dockerfile = "Dockerfile"
  target = "main-onbuild"
}

# Targets to allow injecting customizations from Github Actions.

target "docker-metadata-action-main" {}
target "docker-metadata-action-main-onbuild" {}
