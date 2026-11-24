#! /bin/bash

target=$(mktemp -d)

cleanup() {
    rm -rf "${target}"
}

trap cleanup EXIT

cargo install --path=. --root="${target}"
for f in "${target}"/bin/*; do
    n="$(basename "${f}")"
    echo "Installing ${n}"
    # "${f}" completion zsh > "${XDG_DATA_HOME:-"${HOME}/.local/share"}/zsh/compl/_${n}"
    mv "${f}" "${DOCKER_CONFIG:-"${HOME}/.docker"}/cli-plugins"
done
