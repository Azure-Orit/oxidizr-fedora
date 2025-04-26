#!/usr/bin/env bash

ensure_sudors_installed() {
    ls -la /usr/bin/sudo | MATCH "/usr/bin/sudo -> /usr/bin/sudo-rs"
    ls -la /usr/bin | MATCH ".sudo.oxidizr.bak"
    /usr/bin/sudo --version 2>&1 | MATCH "sudo-rs"

    ls -la /usr/bin/su | MATCH "/usr/bin/su -> /usr/bin/su-rs"
    ls -la /usr/bin | MATCH ".su.oxidizr.bak"
    /usr/bin/su --version 2>&1 | MATCH "su-rs"

    ls -la /usr/bin/visudo | MATCH "/usr/sbin/visudo -> /usr/bin/visudo-rs"
    ls -la /usr/bin | MATCH ".visudo.oxidizr.bak"
}

ensure_sudors_absent() {
    dnf list sudo-rs | NOMATCH installed

    ls -la /usr/bin/sudo | NOMATCH "/usr/bin/sudo -> /usr/bin/sudo-rs"
    ls -la /usr/bin | NOMATCH ".sudo.oxidizr.bak"
    /usr/bin/sudo --version 2>&1 | NOMATCH "sudo-rs"

    ls -la /usr/bin/su | NOMATCH "/usr/bin/su -> /usr/bin/su-rs"
    ls -la /usr/bin | NOMATCH ".su.oxidizr.bak"
    /usr/bin/su --version 2>&1 | NOMATCH "su-rs"

    ls -la /usr/bin/visudo | NOMATCH "/usr/sbin/visudo -> /usr/bin/visudo-rs"
    ls -la /usr/bin | NOMATCH ".visudo.oxidizr.bak"
}
