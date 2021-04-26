#!/bin/sh

set -eux

Mydir="$(dirname -- "$0")"
README="${Mydir}/../README"

comrak "${README}.md" -o "${README}.html"
