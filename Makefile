DOCKER              ?= docker
DOCKER_BUILDER_NAME ?= public-gosh-cli-builder
PLATFORM            ?= linux/amd64,linux/arm64
PROGRESS            ?= linear
# PROGRESS            ?= plain
DOCKER_BUILDX       ?= ${DOCKER} buildx --builder ${DOCKER_BUILDER_NAME} build

.DEFAULT_GOAL := test

TARGET_DIR  := $(abspath $(target-dir))
TARGET_ARCH := "x86_64-unknown-linux-gnu"

GIT_COMMIT := $(shell git rev-parse HEAD)
GIT_BRANCH := $(shell git rev-parse --abbrev-ref HEAD | tr / _)

RELEASE_VERSION :=$(patsubst "%",%,$(strip $(subst version =,,$(shell grep "^version" "./Cargo.toml" | tr -d "'"))))

IMAGE_NAME := teamgosh/gosh-cli
# use current branch if not set
IMAGE_TAG  ?= ${GIT_BRANCH}

# TAG_COMMIT := ${IMAGE_NAME}:${GIT_COMMIT}
FULL_IMAGE_NAME ?= ${IMAGE_NAME}:${RELEASE_VERSION}
TAG_LATEST := ${IMAGE_NAME}:latest

.PHONY: build
build: 
	GOSH_BUILD_VERSION=cargo build --release --bin gosh-cli --target=${TARGET_ARCH}

.PHONY: install
install:
	cargo install --path .

.PHONY: test
test:
	cargo test --release -j 8

.PHONY: qemu
qemu: ## may need to setup qemu
	docker run --privileged --rm tonistiigi/binfmt --install all

.PHONY: prepare-builder
prepare-builder: qemu ## prepare docker buildx builder
	@echo === prepare-builder
	( ${DOCKER} buildx inspect ${DOCKER_BUILDER_NAME} ) || ${DOCKER} buildx create \
		--name ${DOCKER_BUILDER_NAME} \
		${DOCKER_BUILDER_ARGS} \
		--driver docker-container

.PHONY: bench
bench: prepare-builder
	@echo === build + publish
	${DOCKER_BUILDX} \
		--progress=${PROGRESS} \
		--platform ${PLATFORM} \
		-t ${FULL_IMAGE_NAME} \
		${DOCKER_BUILDX_ARGS} \
		\
		-f Dockerfile \
		.

.PHONY: publish
publish: prepare-builder
	@echo === build + publish
	${DOCKER_BUILDX} \
		--push \
		--progress=${PROGRESS} \
		--platform ${PLATFORM} \
		-t ${FULL_IMAGE_NAME} \
		${DOCKER_BUILDX_ARGS} \
		\
		-f Dockerfile \
		.
