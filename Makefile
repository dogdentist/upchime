
all: devops infra services

devops: devops/touch/.devops-rustbuild

devops/touch/.devops-rustbuild: devops/containers/rustbuild/Dockerfile
	docker build \
		--build-arg UID=$(shell id -u) \
		--build-arg GID=$(shell id -g) \
		-t upchime-dev-rustbuild \
		devops/containers/rustbuild/

	@touch devops/touch/.devops-rustbuild

infra: devops/touch/.infra-db-prepare

devops/touch/.infra-db-prepare: source/db-prepare/Dockerfile source/db-prepare/schema.cql source/db-prepare/init.sh
	docker build \
		-t upchime-db-prepare\
		source/db-prepare

	@touch devops/touch/.infra-db-prepare

services: devops/touch/.services-pinger

devops/touch/.services-pinger: source/pinger/Dockerfile $(shell find source/pinger/src -type f -name '*.rs')
	mkdir -p devops/temp/service-pinger-cargo

	docker run \
		-v $(CURDIR)/source/pinger:/home/developer/project \
		-v $(CURDIR)/devops/temp/service-pinger-cargo:/home/developer/.cargo \
		-t upchime-dev-rustbuild \
		cargo build

	docker build \
		-t upchime-pinger \
		source/pinger

	@touch devops/touch/.services-pinger
