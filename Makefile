# Common targets:
#	make build (compile latest release)
#	make init (to init the systemd service on linux)

NAME := entropy-core
SHELL := /bin/bash

default :: show
init :: add-user mkdir copy-service build link

show ::
		cat Makefile

build ::
		cargo build --release

copy-service ::
		cp service/* /etc/systemd/system

add-user ::
		adduser --system --no-create-home --group entropy

mkdir ::
		mkdir -p /var/run/entropy

link ::
		cp target/release/entropy target/release/server /var/run/entropy/
		cp service/* /etc/systemd/system/
		chown entropy:entropy -R /var/run/entropy

# Vercel sets the `HOME` env var weirdly, so we define a few extra
# things to make sure it installs okay.
.PHONY: vercel-rustup
vercel-rustup:
		curl --proto '=https' --tlsv1.2 \
			--silent --show-error --fail https://sh.rustup.rs \
			| RUSTUP_HOME=/vercel/.rustup HOME=/root sh -s -- -y
		cp -R /root/.cargo /vercel/.cargo

# Installs `rustup` in a typical case.
.PHONY: rustup
rustup:
		curl --proto '=https' --tlsv1.2 \
			--silent --show-error --fail https://sh.rustup.rs \
			| sh -s -- -y

.PHONY: rust
rust:
		export PATH="${PATH}:${HOME}/.cargo/bin" rustup default stable \
		&& rustup update nightly \
		&& rustup update stable \
		&& rustup target add wasm32-unknown-unknown --toolchain nightly

# This target is specifically for generating API documentation from
# within a Vercel.com Project. It is used as the Projects `installCommand`.
vercel-install-api-docs :: vercel-rustup rust
		rm -f /vercel/.cargo/bin/rust-analyzer \
			/vercel/.cargo/bin/rustfmt \
			/vercel/.cargo/bin/cargo-fmt \
		# Let's make things even smaller by making it possible to build
		# the `libstd` stuff ourselves.
		export PATH="${PATH}:${HOME}/.cargo/bin" rustup update \
			&& rustup toolchain install nightly-2023-06-15-x86_64-unknown-linux-gnu \
			&& rustup component add rust-src --toolchain nightly-2023-06-15-x86_64-unknown-linux-gnu
		# Install build dependencies required for Amazon Linux 2, the
		# base of the Vercel build image. See:
		# https://vercel.com/docs/concepts/deployments/build-image
		yum install clang-libs clang-devel
		# Manually install Protobuf Compiler `protoc`.
		curl --silent --location \
			https://github.com/protocolbuffers/protobuf/releases/download/v23.4/protoc-23.4-linux-x86_64.zip \
			> /tmp/protoc.zip
		unzip -od /usr /tmp/protoc.zip bin/protoc && rm -f /tmp/protoc.zip
		# Write out SSH deploy keys to prepare cloning private repositories.
		mkdir -p /root/.ssh
		printenv github_ssh_deploy_key_constraints >> /root/.ssh/github_constraints
		printenv github_ssh_deploy_key_synedrion >> /root/.ssh/github_synedrion
		echo "Host github.com*" > /root/.ssh/config
		echo "  Hostname github.com" >> /root/.ssh/config
		echo "	StrictHostKeyChecking no" >> /root/.ssh/config
		echo "Host github.com_constraints" >> /root/.ssh/config
		echo "	IdentityFile /root/.ssh/github_constraints" >> /root/.ssh/config
		echo "Host github.com_synedrion" >> /root/.ssh/config
		echo "	IdentityFile /root/.ssh/github_synedrion" >> /root/.ssh/config
		chmod 600 \
			/root/.ssh/github_synedrion \
			/root/.ssh/github_constraints
		git config --global \
			'url.ssh://git@github.com_constraints/entropyxyz/constraints.git.insteadOf' \
			ssh://git@github.com/entropyxyz/constraints.git
		git config --global \
			'url.ssh://git@github.com_synedrion/entropyxyz/synedrion.git.insteadOf' \
			ssh://git@github.com/entropyxyz/synedrion.git

# The Vercel project's `buildCommand` is defined here.
vercel-build-api-docs ::
		export PATH="${PATH}:${HOME}/.cargo/bin" \
			&& rustup target add wasm32-unknown-unknown \
				--toolchain nightly-2023-06-15-x86_64-unknown-linux-gnu \
			&& cargo +nightly-2023-06-15-x86_64-unknown-linux-gnu doc \
				--profile vercel --no-deps \
				--target x86_64-unknown-linux-gnu \
				-Z build-std=std,panic_abort \
				-Z build-std-features=panic_immediate_abort \
			&& mv /vercel/path0/target/doc/index.html /vercel/path0/target/x86_64-unknown-linux-gnu/doc/
