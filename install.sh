#!/usr/bin/env bash
set -e

function get_arch() {
    a=$(uname -m)
    case ${a} in
    "x86_64" | "amd64")
        echo "amd64"
        ;;
    "i386" | "i486" | "i586")
        echo "386"
        ;;
    "aarch64" | "arm64")
        echo "arm64"
        ;;
    "armv6l" | "armv7l")
        echo "arm"
	;;
    "s390x")
        echo "s390x"
        ;;
    *)
        echo ${NIL}
        ;;
    esac
}

function get_os() {
    echo $(uname -s | awk '{print tolower($0)}')
}

main() {
    local release="0.1.0"
    local os=$(get_os)
    local arch=$(get_arch)
    local dest_file="${HOME}/.uvm/downloads/uvm${release}.${os}-${arch}.tar.gz"
    local url="https://github.com/jinyu/uvm/releases/download/v${release}/uvm${release}.${os}-${arch}.tar.gz"

    echo "[1/3] Downloading ${url}"
    rm -f "${dest_file}"
    if [ -x "$(command -v wget)" ]; then
        wget -q -P "${HOME}/.uvm/downloads" "${url}"
    else
        curl -s -S -L --create-dirs -o "${dest_file}" "${url}"
    fi

    echo "[2/3] Install uvm to the ${HOME}/.uvm/bin"
    mkdir -p "${HOME}/.uvm/bin"
    tar -xz -f "${dest_file}" -C "${HOME}/.uvm/bin"
    chmod +x "${HOME}/.uvm/bin/uvm"

    echo "[3/3] Set environment variables"
    cat >${HOME}/.uvm/env <<-'EOF'
#!/bin/sh
# uvm shell setup
export PATH="${HOME}/.uvm/bin:$PATH"
	EOF


    if [ -x "$(command -v bash)" ]; then
        cat >>${HOME}/.bashrc <<-'EOF'
if [ -f "${HOME}/.uvm/env" ]; then
    . "${HOME}/.uvm/env"
fi
		EOF
    fi

    if [ -x "$(command -v zsh)" ]; then
        cat >>${HOME}/.zshrc <<-'EOF'
if [ -f "${HOME}/.uvm/env" ]; then
    . "${HOME}/.uvm/env"
fi
		EOF
    fi


    echo -e "\nTo configure your current shell, run:\nsource \"$HOME/.uvm/env\""

    exit 0
}

main
