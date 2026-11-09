#!/bin/sh
# Why this file exists
# --------------------
# Craft Anything checks a build by starting `cargo build` from a folder that can
# be gone by the time the compiler starts, with a PATH that holds only the Rust
# tools. rustc inherits that folder as its working directory, finds it missing,
# and refuses to run:
#
#   error: failed to run `rustc` to learn about target-specific information
#     error: Current directory is invalid: No such file or directory (os error 2)
#
# .cargo/config.toml names this script as the compiler. It steps into the app
# folder first — the folder this script lives in, one level up, so it keeps
# working if the project is moved or copied — and only then starts the real
# compiler with the arguments unchanged.
#
# Two rules this file follows:
#  - Shell built-ins only. Under the build check's stripped PATH there is no
#    dirname, pwd, sleep or command to call (the first version of this script
#    used dirname and failed every time the check ran).
#  - Never hand over while the working directory is invalid: a missing one is
#    repaired before rustc is started, and PATH is put back to something normal
#    so the link step can find the C compiler and the assembler.

case "$0" in
    */*) shim_dir=${0%/*} ;;
    *)   shim_dir=. ;;
esac
app_dir=$shim_dir/..

# Remember whether the working directory was still there: if it was not, that is
# worth one line in .cargo/shim.log, because it is the failure this file exists
# to prevent.
note=
[ -d . ] || note="invalid cwd"

PATH=/usr/local/bin:/usr/bin:/bin:/usr/local/sbin:/usr/sbin:/sbin${PATH:+:$PATH}
export PATH
[ -n "${HOME:-}" ] || HOME=/home/zital
export HOME

if ! cd "$app_dir" 2>/dev/null; then
    note="${note:+$note, }no app folder at $app_dir"
    if ! cd "${PWD:-/}" 2>/dev/null; then
        cd / 2>/dev/null
    fi
fi

if [ -n "$note" ]; then
    # One line, once: enough to show this file had to step in, without a log
    # that grows with every compiler call of a long build.
    [ -e "$app_dir/.cargo/shim.log" ] || printf '%s: %s\n' "$$" "$note" >> "$app_dir/.cargo/shim.log" 2>/dev/null
fi

real_rustc=/home/zital/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustc
if [ ! -x "$real_rustc" ]; then
    for d in /home/zital/.cargo/bin /usr/local/bin /usr/bin; do
        if [ -x "$d/rustc" ]; then
            real_rustc=$d/rustc
            break
        fi
    done
fi
[ -x "$real_rustc" ] || { printf 'rustc-shim: no rustc found\n' >&2; exit 127; }

exec "$real_rustc" "$@"
