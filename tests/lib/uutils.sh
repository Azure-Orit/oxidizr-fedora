#!/usr/bin/env bash

ensure_coreutils_installed() {
    dnf list uutils-coreutils | MATCH installed
    while read p; do
        util_path="$(which $p)"
        util_dir="$(dirname "$util_path")"
        ls -la "$util_path"| MATCH "$util_path -> /usr/bin/coreutils"
        ls -la "$util_dir/.$p.oxidizr.bak" || echo "No backup file for $util_path"
        $util_path --help | NOMATCH "https://www.gnu.org/software/coreutils"
    done < ${SPREAD_PATH}/tests/lib/rust-coreutils-bins.txt
}

ensure_coreutils_absent() {
    dnf list uutils-coreutils | NOMATCH installed
    ls -la /usr/bin/date | NOMATCH "/usr/bin/date -> /usr/bin/coreutils"
    ls -la /usr/bin | NOMATCH ".date.oxidizr.bak"
    date --help | MATCH "GNU"
}
