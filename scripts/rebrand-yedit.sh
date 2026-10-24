#!/bin/sh
# Rename a clean PhotoCraft checkout to YEDIT: drop the ArtCraft marks, move every branded path and
# rewrite the product, crate, id and native-format names in text files. Mechanical only: the links,
# update check, installer ids, icon, NOTICE and README are edited by hand afterwards.
#
# Kept verbatim (attribution and upstream references must stay true): license texts, NOTICE,
# "PhotoCraft contributors", "the PhotoCraft authors", storytold/photocraft issue and PR links,
# the photocraft-corpus test repository and getartcraft.com/apps/photocraft.
#
# Binary files keep the old names inside; regenerate the ones that tests compare with the code:
#   examples/plugins/invert-rs/build.sh                      (crates/plugins/tests/fixtures/invert.wasm)
#   YEDIT_REGEN_PROFILES=1 cargo test -p yedit-cms --release --test it -- regen::   (cms/profiles/*.icc)
#   packaging/icons.sh                                       (assets/app-icon/*, from the new yedit.svg)
# packaging/macos/dmg/dmg-layout.DS_Store still names the PhotoCraft volume (macOS packaging only).
set -eu
cd "$(dirname "$0")/.."

APP_ID=com.fesilabs.yedit
SELF=scripts/rebrand-yedit.sh

if [ -n "$(git status --porcelain -- . ":(exclude)$SELF")" ]; then
    echo "rebrand-yedit: the working tree has changes; commit or stash them first" >&2
    exit 1
fi

# The ArtCraft name, wordmark and logos are not open source; forks must remove them
# (docs/brand/LICENSE-brand.txt).
if [ -d docs/brand ]; then
    git rm -r -q docs/brand
fi

export APP_ID
REWRITE='
BEGIN {
    @keep = (
        "ArtCraft Team and the PhotoCraft contributors",
        "PhotoCraft contributors",
        "the PhotoCraft authors",
        "storytold/photocraft",
        "getartcraft.com/apps/photocraft",
        "photocraft-corpus",
    );
}
for my $i (0 .. $#keep) { my $k = quotemeta $keep[$i]; s/$k/\x00KEEP$i\x00/g; }
s/ai\.storyteller\.photocraft/$ENV{APP_ID}/g;
s/PhotoCraft/YEDIT/g; s/PHOTOCRAFT/YEDIT/g; s/Photocraft/Yedit/g; s/photocraft/yedit/g;
s/PCRAFT/YEDIT/g;     s/Pcraft/Yedit/g;     s/pcraft/yedit/g;
s/PCTIP1/YETIP1/g;     s/pc(droplet|brushes|presets|save|tip)/y$1/g;
s/\x00KEEP(\d+)\x00/$keep[$1]/g;
'

# Paths first, file by file (git has no directory moves).
git ls-files | grep -E 'photocraft|PhotoCraft|pcraft' | while IFS= read -r old; do
    new=$(printf '%s\n' "$old" | perl -pe "$REWRITE")
    mkdir -p "$(dirname "$new")"
    git mv "$old" "$new"
    rmdir -p "$(dirname "$old")" 2>/dev/null || true
done

# Then contents of every tracked text file, except license texts and this script.
git grep -z -l -I -E 'photocraft|PhotoCraft|Photocraft|PHOTOCRAFT|pcraft|Pcraft|PCRAFT|ai\.storyteller\.photocraft|PCTIP1|pc(droplet|brushes|presets|save|tip)' -- . \
    ':(exclude)LICENSE-MIT' ':(exclude)LICENSE-APACHE' ':(exclude)NOTICE' ":(exclude)$SELF" \
    ':(exclude,glob)**/LICENSE*.txt' ':(exclude,glob)**/OFL*.txt' |
    xargs -0 perl -i -pe "$REWRITE"

echo "rebrand-yedit: done. Left for review:"
git grep -n -I -E 'PhotoCraft|photocraft|ArtCraft|getartcraft|storyteller' -- . \
    ':(exclude)LICENSE-MIT' ':(exclude)LICENSE-APACHE' ':(exclude)NOTICE' ":(exclude)$SELF" \
    ':(exclude,glob)**/LICENSE*.txt' ':(exclude,glob)**/OFL*.txt' | wc -l
