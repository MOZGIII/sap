group "default" {
  targets = ["sap", "sap-onbuild"]
}

target "sap" {
  inherits = ["docker-metadata-action-sap"]
  dockerfile = "Dockerfile"
  target = "sap"
}

target "sap-onbuild" {
  inherits = ["docker-metadata-action-sap-onbuild"]
  dockerfile = "Dockerfile"
  target = "sap-onbuild"
}

# Targets to allow injecting customizations from Github Actions.

target "docker-metadata-action-sap" {}
target "docker-metadata-action-sap-onbuild" {}
